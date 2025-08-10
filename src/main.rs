use anyhow::Result;
use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use reqwest::{blocking, header};
use rustyline::DefaultEditor;
use serde::Serialize;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

mod archive_formatter;
mod input_parser;
mod types;

use types::{Archive, Card};

const SCRYFALL_API_ROOT: &str = "https://api.scryfall.com/";

fn main() -> Result<()> {
    let args = Options::parse();

    match args.subcommand {
        Some(Commands::Export {
            output,
            input,
            format,
        }) => command_export(input, output, format)?,
        Some(Commands::Add { set_code, output }) => command_add(set_code, output)?,
        Some(Commands::CollectionPath) => println!("{}", archive_collection_path().display()),
        _ => {}
    }

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(arg_required_else_help = true)]
#[command(name = "Crackathon")]
struct Options {
    #[arg(short, long)]
    pub debug: Option<bool>,
    #[command(subcommand)]
    pub subcommand: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Export a collection to a file to be consumed by other tools.
    Export {
        /// Which file to write to.
        #[arg(short, long, value_name = "OUTPUT_FILE")]
        output: Option<PathBuf>,
        /// Which file to read from.
        #[arg(short, long, value_name = "INPUT_FILE")]
        input: Option<PathBuf>,
        /// Export as either Decklist, or CSV format.
        #[arg(short, long, value_enum)]
        format: Option<ExportType>,
    },
    /// Add some cards to a collection.
    Add {
        /// Set code to assume for additions.
        set_code: Option<String>,
        /// Output file to use. Use this to maintain separate lists, for e.g.
        /// decks.
        #[arg(short, long, value_name = "OUTPUT_FILE")]
        output: Option<PathBuf>,
    },
    /// Dump the default collection path. Useful for scripting.
    CollectionPath,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ExportType {
    Deck,
    Csv,
}

fn command_add(set_code: Option<String>, output: Option<PathBuf>) -> Result<()> {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/json".parse().unwrap());
    let client = blocking::ClientBuilder::new()
        .user_agent("Crack-a-thon, see github.com/MordecaiMalignatus/archivist.")
        .default_headers(headers)
        .build()?;
    let mut rl = DefaultEditor::new()?;

    loop {
        let buffer = rl.readline("Enter Card Number: ")?;
        let buffer = buffer.trim().to_string();
        rl.add_history_entry(buffer.as_str())?;

        if buffer.is_empty() {
            println!("Empty input received, exiting...");
            break;
        }
        let parsed_input = match input_parser::parse_addition_input(buffer, set_code.clone()) {
            Ok(parsed) => parsed,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        let mut card =
            query_scryfall_for_card(&parsed_input.set_code, &parsed_input.card_number, &client)?;
        card.foil = parsed_input.foil;

        let resulting_count = edit_archive(card.clone(), output.clone(), parsed_input.removal)?;
        let modification_text = match parsed_input.removal {
            true => match resulting_count {
                0 => format!("Removed {} from collection!\n", card.name),
                _ => format!(
                    "Removed {} from collection! ({resulting_count} remaining in this collection)\n",
                    card.name
                ),
            },
            false => {
                let price_string = match card.prices {
                    Some(prices) => match card.foil {
                        true => {
                            // TODO(sar): this is not always true, mis-input happens, I should care for that right.
                            format!(
                                "({}€ / ${})",
                                prices.eur_foil.unwrap(),
                                prices.usd_foil.unwrap()
                            )
                        }
                        false => format!("({}€ / ${})", prices.eur, prices.usd),
                    },
                    None => "".to_string(),
                };
                match resulting_count {
                    1 => format!("Added {} to collection! {price_string}\n", card.name),
                    c => format!(
                        "Added {} to collection! ({c} in this collection) {price_string}\n",
                        card.name
                    ),
                }
            }
        };

        println!("{modification_text}")
    }
    Ok(())
}

/// Export converts the current collection to the common format that is accepted
/// by Arena, Moxfield et al. This format is roughly:
/// "$AMOUNT $CARDNAME ($SETCODE)? $NUMBER? $FOIL?"
/// Due to the internal structure of this application, the export is going to be sorted by set.
fn command_export(
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    format: Option<ExportType>,
) -> Result<()> {
    let Archive(a) = read_collection(input_path)?;
    let output = match format {
        Some(ExportType::Csv) => format_as_moxfield_csv(&a),
        Some(ExportType::Deck) => format_as_deck_list(&a),
        None => format_as_deck_list(&a),
    };

    match output_path {
        Some(path) => {
            std::fs::write(path.clone(), output)?;
            println!("Wrote output to {}", path.display())
        }
        None => println!("{output}"),
    }

    Ok(())
}

/// Exports the decklist as a moxfield-compatible CSV. Documentation can be
/// found here: https://moxfield.com/help/importing-collection
fn format_as_moxfield_csv(archive: &HashMap<String, Vec<Card>>) -> String {
    let mut output = String::new();
    output.push_str("\"Count\",\"Name\",\"Collector Number\",\"Edition\",\"Foil\"\n");
    archive.iter().for_each(|(_set, v)| {
        v.iter().for_each(|card| {
            let line = format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                card.count,
                card.name,
                card.collector_number,
                card.set,
                if card.foil { "foil" } else { "" },
            );
            output.push_str(&line);
        });
    });

    output
}

fn format_as_deck_list(archive: &HashMap<String, Vec<Card>>) -> String {
    let mut output = String::new();
    archive.iter().for_each(|(_set, v)| {
        v.iter().for_each(|card| {
            let line = format!(
                "{} {} ({}) {} {}\n",
                card.count,
                card.name,
                card.set.to_ascii_uppercase(),
                card.collector_number,
                if card.foil { "*F*" } else { "" }
            );
            output.push_str(&line);
        });
    });

    output
}

/// Adds `c` to the archive specified at `path`, if not, the default collection.
/// Returns either the amount of cards now present in the collection, or an
/// error.
fn edit_archive(c: Card, path: Option<PathBuf>, removal: bool) -> Result<usize> {
    let Archive(mut a) = read_collection(path.clone())?;
    let count = if a.contains_key(&c.set) {
        let set_list = a
            .get_mut(&c.set)
            .expect("didn't find sub-list despite checking for presence prior");
        match removal {
            true => remove_or_decrement(c, set_list)?,
            false => add_or_increment(c, set_list)?,
        }
    } else {
        match removal {
            true => {
                return Err(anyhow!(
                    "Can't remove card from collection, because no copies are in the collection."
                ));
            }
            false => {
                a.insert(c.set.clone(), vec![c]);
                1
            }
        }
    };

    let file_content = serialize_with_formatter(&mut a)?;
    let _ = match path {
        Some(p) => std::fs::write(p, file_content),
        None => std::fs::write(archive_collection_path(), file_content),
    };

    Ok(count)
}

fn serialize_with_formatter(input: &mut HashMap<String, Vec<Card>>) -> Result<Vec<u8>> {
    let mut file_content = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(
        &mut file_content,
        archive_formatter::ArchiveFormatter::new(),
    );
    input
        .serialize(&mut serializer)
        .map_err(|err| anyhow!("error when serializing archive to string: {err}"))?;

    Ok(file_content)
}

fn add_or_increment(c: Card, set_list: &mut Vec<Card>) -> Result<usize> {
    let position = set_list
        .iter()
        .position(|owned_card| owned_card.name == c.name && owned_card.foil == c.foil);

    let count: usize = match position {
        Some(i) => {
            set_list[i].count += 1;
            set_list[i].count as usize
        }
        None => {
            set_list.push(c);
            1
        }
    };
    set_list.sort_by_key(|c| c.collector_number.clone());

    Ok(count)
}

fn remove_or_decrement(c: Card, set_list: &mut Vec<Card>) -> Result<usize> {
    let position = set_list
        .iter()
        .position(|owned_card| owned_card.name == c.name && owned_card.foil == c.foil);

    let count: usize = match position {
        Some(i) => {
            if set_list[i].count == 1 {
                set_list.remove(i);
                0
            } else {
                set_list[i].count -= 1;
                set_list[i].count as usize
            }
        }
        None => return Err(anyhow!("No card present in collection, can't remove it.")),
    };
    set_list.sort_by_key(|c| c.collector_number.clone());

    Ok(count)
}

fn query_scryfall_for_card(
    set: &str,
    number: &str,
    client: &reqwest::blocking::Client,
) -> Result<Card> {
    let url = reqwest::Url::parse(&format!("{SCRYFALL_API_ROOT}/cards/{set}/{number}"))?;
    let req = client.get(url).build()?;
    let res = client.execute(req)?;
    if res.status() != 200 {
        return Err(anyhow!(
            "Error from Scryfall, response: {}",
            res.text().unwrap()
        ));
    }
    let mut card = res.json::<Card>()?;
    card.count = 1;
    card.foil = false;
    Ok(card)
}

fn read_collection(explicit_path: Option<PathBuf>) -> Result<Archive> {
    let path = match explicit_path {
        Some(path) => path,
        None => archive_collection_path(),
    };
    let file = match std::fs::read_to_string(path) {
        Ok(res) => res,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    // If it's not found, return an empty JSON object that is going
                    // to parse into an empty HashMap just fine.
                    "{}".to_string()
                }
                _ => return Err(anyhow!("Could not read archive: {e}")),
            }
        }
    };
    match serde_json::from_str(&file) {
        Ok(archive) => Ok(archive),
        Err(e) => Err(anyhow!("Archive is not valid JSON: {e}")),
    }
}

fn archive_path() -> PathBuf {
    let homedir = env::home_dir().expect("Can't get user home directory");
    let config_folder = homedir.join(".config").join("crack");
    match fs::create_dir_all(&config_folder) {
        Ok(_) => config_folder,
        Err(e) => panic!("can't create folder at {}: {}", config_folder.display(), e),
    }
}

fn archive_collection_path() -> PathBuf {
    archive_path().join("collection.json")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_foil_addition() {
        let mut set_list = Vec::<Card>::new();
        let mut c1 = Card::default();
        let mut c2 = Card::default();
        c1.name = String::from("a card");
        c2.name = String::from("a card");
        c2.foil = true;

        add_or_increment(c1, &mut set_list).unwrap();
        assert_eq!(set_list.len(), 1);

        add_or_increment(c2, &mut set_list).unwrap();
        assert_eq!(set_list.len(), 2);
    }

    #[test]
    fn test_duplicate_addition() {
        let mut set_list = Vec::<Card>::new();
        let mut c1 = Card::default();
        let mut c2 = Card::default();
        c1.name = String::from("a card");
        c2.name = String::from("a card");

        add_or_increment(c1, &mut set_list).unwrap();
        assert_eq!(set_list.len(), 1);

        add_or_increment(c2, &mut set_list).unwrap();
        assert_eq!(set_list.len(), 1);
    }
}

use anyhow::Result;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use reqwest::{blocking, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::{env, io};

mod archive_formatter;
mod input_parser;

const SCRYFALL_API_ROOT: &str = "https://api.scryfall.com/";

fn main() -> Result<()> {
    let args = Options::parse();
    let stdin = io::stdin();
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/json".parse().unwrap());
    let client = blocking::ClientBuilder::new()
        .user_agent("Crack-a-thon, see github.com/MordecaiMalignatus/archivist.")
        .default_headers(headers)
        .build()?;

    match args.subcommand {
        Some(Commands::Export { output, input }) => command_export(input, output)?,
        Some(Commands::Add { set_code, output }) => loop {
            print!("Enter card number: ");
            let mut buffer = String::new();
            let _ = stdin.read_line(&mut buffer)?;
            let buffer = buffer.trim().to_string();

            if buffer.is_empty() {
                break;
            }
            let parsed_input = match input_parser::parse_addition_input(buffer, set_code.clone()) {
                Ok(parsed) => parsed,
                Err(e) => {
                    eprintln!("{e}");
                    continue;
                }
            };

            let mut card = get_card(&parsed_input.set_code, &parsed_input.card_number, &client)?;
            card.foil = parsed_input.foil;

            match add_to_archive(card.clone(), output.clone())? {
                1 => println!("Added {} to collection!\n", card.name),
                count => println!(
                    "Added {} to collection! ({count} in collection)\n",
                    card.name
                ),
            };
        },
        _ => panic!("must supply subcommand"),
    }

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(name = "Crack")]
struct Options {
    #[arg(short, long)]
    pub debug: Option<bool>,
    #[command(subcommand)]
    pub subcommand: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Export {
        #[arg(short, long, value_name = "OUTPUT_FILE")]
        output: Option<PathBuf>,
        #[arg(short, long, value_name = "INPUT_FILE")]
        input: Option<PathBuf>,
    },
    Add {
        set_code: Option<String>,
        #[arg(short, long, value_name = "OUTPUT_FILE")]
        output: Option<PathBuf>,
    },
}

/// Export converts the current collection to the common format that is accepted
/// by Arena, Moxfield et al. This format is roughly:
/// "$AMOUNT $CARDNAME ($SETCODE)? $NUMBER? $FOIL?"
/// Due to the internal structure of this application, the export is going to be sorted by set.
fn command_export(input_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<()> {
    let Archive(a) = read_archive(input_path)?;
    let mut output = String::new();
    a.into_iter().for_each(|(_set, v)| {
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

    match output_path {
        Some(path) => {
            std::fs::write(path.clone(), output)?;
            println!("Wrote output to {}", path.display())
        }
        None => println!("{output}"),
    }

    Ok(())
}

/// Adds `c` to the archive specified at `path`, if not, the default collection.
/// Returns either the amount of cards now present in the collection, or an
/// error.
fn add_to_archive(c: Card, path: Option<PathBuf>) -> Result<usize> {
    let Archive(mut a) = read_archive(path.clone())?;
    let count = if a.contains_key(&c.set) {
        let set_list = a
            .get_mut(&c.set)
            .expect("didn't find sub-list despite checking for presence prior");
        add_or_increment(c, set_list)?
    } else {
        a.insert(c.set.clone(), vec![c]);
        1
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
    Ok(count)
}

fn get_card(set: &str, number: &str, client: &reqwest::blocking::Client) -> Result<Card> {
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

fn read_archive(explicit_path: Option<PathBuf>) -> Result<Archive> {
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
    let config_folder = homedir.join("crack");
    match fs::create_dir_all(&config_folder) {
        Ok(_) => config_folder,
        Err(e) => panic!("can't create folder at {}: {}", config_folder.display(), e),
    }
}

fn archive_collection_path() -> PathBuf {
    archive_path().join("collection.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Card {
    pub name: String,
    pub collector_number: String,
    pub set_name: String,
    pub oracle_id: String,
    #[serde(default)]
    pub count: u32,
    pub colors: Vec<String>,
    pub rarity: String,
    pub uri: String,
    pub set: String,
    pub foil: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Archive(HashMap<String, Vec<Card>>);

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

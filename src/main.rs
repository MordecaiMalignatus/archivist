use anyhow::Result;
use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use reqwest::{blocking, header};
use rustyline::DefaultEditor;
use types::OldArchive;
use types::State;

use std::env;
use std::fs;
use std::path::PathBuf;

mod input_parser;
mod scryfall;
mod types;

use types::{Archive, Card};

fn main() -> Result<()> {
    let args = Options::parse();

    match args.subcommand {
        Some(Commands::Export {
            output,
            input,
            format,
        }) => command_export(input, output, format)?,
        Some(Commands::Add {
            output_file,
            set_code,
        }) => command_add(output_file, set_code)?,
        Some(Commands::CollectionPath) => println!("{}", archive_collection_path().display()),
        //        Some(Commands::Search { path }) => command_search(path)?,
        Some(Commands::Create { name, set_used }) => command_list_create(name, set_used)?,
        Some(Commands::List { subcommand }) => match subcommand {
            ListCommands::Create { name, set_used } => command_list_create(name, set_used)?,
            ListCommands::Use { path } => command_list_use(path)?,
        },
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
        /// Output file to use. If not specified, entered cards modify the default collection.
        #[arg()]
        output_file: Option<PathBuf>,
        /// Set code to default to. Very useful when entering boosters.
        #[arg(short, long, value_name = "SET_CODE")]
        set_code: Option<String>,
    },
    /// Dump the default collection path. Useful for scripting.
    CollectionPath,
    /// Search the specified collection.
    Search {
        #[arg()]
        path: Option<PathBuf>,
    },
    /// Manipulate decklists and collections.
    List {
        #[command(subcommand)]
        subcommand: ListCommands,
    },
    /// Create a new deck list. Optionally, set as current list. Alias from the `list create` subcommand.
    Create {
        /// Deck name. Used for the filename, as well as the display name.
        #[arg(short, long, value_name = "DECK_NAME")]
        name: String,
        /// Whether or not to set the decklist as the currently active default collection. Defaults to true.
        #[arg(short, long, default_value = "true")]
        set_used: bool,
    },
    // /// Change the crackathon configuration.
    // Config {
    //     #[arg(long)]
    //     set_home: String,
    // },
}

#[derive(Subcommand)]
enum ListCommands {
    /// Create a new deck list. Optionally, set as current list.
    Create {
        /// Deck name. Used for the filename, as well as the display name.
        #[arg(short, long, value_name = "DECK_NAME")]
        name: String,
        /// Whether or not to set the decklist as the currently active default collection. Defaults to true.
        #[arg(short, long, default_value = "true")]
        set_used: bool,
    },
    /// Set a new list as "current". Opens a selector if not given a path.
    Use {
        #[arg(value_name = "DECK_NAME")]
        path: Option<String>,
    },
    // /// Delete a decklist. Opens a selector if not given a path.
    // Delete {
    //     #[arg(value_name = "DECK_PATH")]
    //     path: Option<String>,
    // },
    // /// Prints a decklist. Prints currently used decklist if not given a path.
    // /// Optionally opens selector.
    // Show {
    //     #[arg(value_name = "DECK_PATH")]
    //     path: Option<String>,
    //     #[arg(short, long)]
    //     select: bool,
    // },
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ExportType {
    Deck,
    Csv,
}

fn command_add(output: Option<PathBuf>, set_code: Option<String>) -> Result<()> {
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

        let mut card = match scryfall::query_card(
            &parsed_input.set_code,
            &parsed_input.card_number,
            &client,
        ) {
            Ok(card) => card,
            Err(e) => {
                eprintln!("Error from scryfall: {e}");
                continue;
            }
        };
        card.foil = parsed_input.foil;

        let resulting_count = match edit_archive(card.clone(), output.clone(), parsed_input.removal)
        {
            Ok(i) => i,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };
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
/// by Arena, Moxfield et al. This format is roughly: "$AMOUNT $CARDNAME
/// ($SETCODE)? $NUMBER? $FOIL?" Due to the internal structure of this
/// application, the export is going to be sorted by set.
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
fn format_as_moxfield_csv(archive: &[Card]) -> String {
    let mut output = String::new();
    output.push_str("\"Count\",\"Name\",\"Collector Number\",\"Edition\",\"Foil\"\n");

    archive.iter().for_each(|card| {
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

    output
}

fn format_as_deck_list(archive: &[Card]) -> String {
    let mut output = String::new();

    archive.iter().for_each(|card| {
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

    output
}

fn command_list_create(name: String, _set_used: bool) -> Result<()> {
    let root = archive_path();
    let root = root.join(format!("{name}.json"));
    let Archive(empty_archive) = Archive::default();

    let file_content = serde_json::to_string_pretty(&empty_archive)?;
    let _ = std::fs::write(root.clone(), file_content);

    println!("Created new list at {}", root.display());
    Ok(())
}

fn command_list_use(name: Option<String>) -> Result<()> {
    let mut state = read_state()?;

    match name {
        Some(specified_name) => {
            // TODO(sar): Check for ENOENT here
            let old_deck = state.currently_used_deck;
            state.currently_used_deck = Some(specified_name.clone());
            match old_deck {
                Some(old_deck) => println!("Changed used deck from {old_deck} to {specified_name}"),
                None => println!("Changed used deck from the collection to {specified_name}"),
            }
        }
        None => {
            state.currently_used_deck = None;
            println!("Unset current deck, defaulting back to the collection.");
        }
    }

    // if absolute path, use that
    // if bare string, use format!("{config_dir()}.{name}.json)`
    // if empty/None, open `skim`
    // if not found, offer to create

    write_state(state)
}

/// Adds `c` to the archive specified at `path`, if not, the deck specified in
/// the state, if not that, the default collection. Returns either the amount of
/// cards now present in the collection, or an error.
fn edit_archive(c: Card, path: Option<PathBuf>, removal: bool) -> Result<u32> {
    let Archive(mut a) = read_collection(path.clone())?;
    let card_in_archive = a.iter_mut().find(|archive_card| {
        archive_card.set == c.set
            && archive_card.collector_number == c.collector_number
            && archive_card.foil == c.foil
    });

    let count = match card_in_archive {
        Some(archive_card) => {
            if removal {
                archive_card.count -= 1;
            } else {
                archive_card.count += 1;
            }
            archive_card.count
        }
        None => {
            if removal {
                return Err(anyhow!(
                    "Can't remove card from collection, because no copies are in the collection."
                ));
            }
            a.push(c.clone());
            c.count
        }
    };
    let file_content = serde_json::to_string_pretty(&a)?;
    write_collection(file_content, path)?;
    Ok(count)
}

fn read_collection(explicit_path: Option<PathBuf>) -> Result<Archive> {
    let path = match explicit_path {
        Some(path) => path,
        None => default_collection_path()?,
    };
    let file = match std::fs::read_to_string(path) {
        Ok(res) => res,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    // If it's not found, return an empty JSON object that is going
                    // to parse into an empty List just fine.
                    "[]".to_string()
                }
                _ => return Err(anyhow!("Could not read archive: {e}")),
            }
        }
    };
    let res: Result<Archive> = match serde_json::from_str(&file) {
        Ok(res) => return Ok(res),
        _ => {
            let fallback_res: OldArchive = match serde_json::from_str(&file) {
                Ok(res) => res,
                Err(_) => {
                    return Err(anyhow!(
                        "Archive is in an invalid format, neither current nor past archive formats parse."
                    ));
                }
            };
            let combined_list = fallback_res
                .0
                .into_values()
                .reduce(|acc, el| acc.into_iter().chain(el).collect())
                .unwrap();

            Ok(Archive(combined_list))
        }
    };
    res
}

fn write_collection(content: String, explicit_path: Option<PathBuf>) -> Result<()> {
    let path = match explicit_path {
        Some(path) => path,
        None => default_collection_path()?,
    };

    std::fs::write(path, content)?;
    Ok(())
}

fn default_collection_path() -> Result<PathBuf> {
    let state = read_state()?;
    let res = match state.currently_used_deck {
        Some(deck_name) => archive_path().join(format!("{deck_name}.json")),
        None => archive_collection_path(),
    };
    Ok(res)
}

fn read_state() -> Result<State> {
    let file = match std::fs::read_to_string(state_file_path()) {
        Ok(res) => res,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    // If it's not found, return an empty JSON object that is going
                    // to parse into an empty HashMap just fine.
                    "{}".to_string()
                }
                _ => return Err(anyhow!("Could not read statefile: {e}")),
            }
        }
    };
    match serde_json::from_str(&file) {
        Ok(archive) => Ok(archive),
        Err(e) => Err(anyhow!("Archive is not valid JSON: {e}")),
    }
}

fn write_state(s: State) -> Result<()> {
    let file_content = serde_json::to_string_pretty(&s)?;
    std::fs::write(state_file_path(), file_content)?;
    Ok(())
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

fn state_file_path() -> PathBuf {
    archive_path().join("_state.json")
}

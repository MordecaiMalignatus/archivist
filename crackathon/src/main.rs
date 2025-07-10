use anyhow::Result;
use anyhow::anyhow;
use reqwest::{blocking, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, io};

mod human_readable_formatter;

const SCRYFALL_API_ROOT: &str = "https://api.scryfall.com/";

fn main() -> Result<()> {
    let set_code = "blb".to_string();
    let stdin = io::stdin();
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/json".parse().unwrap());
    let client = blocking::ClientBuilder::new()
        .user_agent("Crack-a-thon, see github.com/MordecaiMalignatus/archivist.")
        .default_headers(headers)
        .build()?;

    loop {
        println!("Enter card number: ");
        let mut buffer = String::new();
        let _ = stdin.read_line(&mut buffer)?;
        let buffer = buffer.trim();
        if buffer.is_empty() {
            break;
        }
        let number = buffer.parse::<u32>()?;
        let card = get_card(&set_code, number, &client)?;
        add_to_archive(card.clone())?;
        println!("Added {} to collection!", card.name)
    }

    Ok(())
}

fn add_to_archive(c: Card) -> Result<()> {
    let Archive(mut a) = read_archive()?;
    if a.contains_key(&c.set) {
        let set_list = a
            .get_mut(&c.set)
            .expect("didn't find sub-list despite checking for presence prior");
        add_or_increment(c, set_list)?
    } else {
        a.insert(c.set.clone(), vec![c]);
    }

    let mut file_content = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(
        &mut file_content,
        human_readable_formatter::HumanReadableFormatter::new(),
    );
    a.serialize(&mut serializer)
        .map_err(|err| anyhow!("error when serializing archive to string: {err}"))?;

    let _ = std::fs::write(archive_path(), file_content);

    Ok(())
}

fn add_or_increment(c: Card, set_list: &mut Vec<Card>) -> Result<()> {
    let position = set_list
        .iter()
        .position(|owned_card| owned_card.name == c.name);

    match position {
        Some(i) => set_list[i].count += 1,
        None => set_list.push(c),
    }
    Ok(())
}

fn get_card(set: &str, number: u32, client: &reqwest::blocking::Client) -> Result<Card> {
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
    Ok(card)
}

fn read_archive() -> Result<Archive> {
    let path = archive_path();
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
    homedir.join("mtg-archive.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Card {
    pub name: String,
    pub set_name: String,
    pub oracle_id: String,
    #[serde(default)]
    pub count: u32,
    pub colors: Vec<String>,
    pub rarity: String,
    pub uri: String,
    pub set: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Archive(HashMap<String, Vec<Card>>);

use anyhow::Result;
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::PathBuf};

const SCRYFALL_API_ROOT: &str = "https://api.scryfall.com/";

fn main() {
    println!("Hello, world!");
    let set_code = "blb".to_string();

    loop {
        let input = getline("Enter card ID: ");
        if input == "" {
            break;
        }
        let number = input.parse::<u32>();
        let card = get_card(set_code, number)?;
        add_to_archive(card)?
    }
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

fn get_card(set: String, number: i32) -> Result<Card> {
    let url = reqwest::Url::parse(&format!("{SCRYFALL_API_ROOT}/cards/{set}/{number}"))?;
    let mut res = blocking::get(url)?.json::<Card>()?;
    res.count = 1;
    Ok(res)
}

fn read_archive() -> Result<Archive> {
    let path = archive_path();
    let file = std::fs::read_to_string(path).expect("Can't read archive");
    match serde_json::from_str(&file) {
        Ok(archive) => Ok(archive),
        Err(e) => panic!("Archive is not valid JSON: {e}"),
    }
}

fn archive_path() -> PathBuf {
    let homedir = env::home_dir().expect("Can't get user home directory");
    homedir.join("mtg-archive.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Card {
    pub oracle_id: String,
    pub uri: String,
    pub name: String,
    pub set: String,
    pub set_name: String,
    pub rarity: String,
    #[serde(default)]
    pub count: u32,
    pub colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Archive(HashMap<String, Vec<Card>>);

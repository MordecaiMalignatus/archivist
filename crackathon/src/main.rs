use std::collections::HashMap;
use anyhow::Result;

fn main() {
    println!("Hello, world!");
}

fn get_card(set: String, number: i32) ->  Result<Card> { todo!() }

struct Card {
    pub oracle_id: String,
    pub uri: String,
    pub name: String,
    pub set: String,
    pub set_name: String,
    pub rarity: String,
    pub count: u32,
}

struct Archive (HashMap<String, Vec<Card>>);

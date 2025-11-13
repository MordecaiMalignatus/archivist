use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A Scryfall card object, reduced by quite a few fields. The API docs for the
/// full struct can be found here: https://scryfall.com/docs/api/cards/collector
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Card {
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
    pub prices: Option<CardPrices>,
}

/// Small embedded struct that captures the pricing information returned by Scryfall.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardPrices {
    pub usd: String,
    pub usd_foil: Option<String>,
    pub eur: String,
    pub eur_foil: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Archive(pub Vec<Card>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OldArchive(pub HashMap<String, Vec<Card>>);


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    /// A string naming a deck. This is not intended to be an absolute path as
    /// the deck home can change, but rather the bits between home path and
    /// `.json`, ie `/some/home/path/.config/crack/_statefile_.json`.
    pub currently_used_deck: Option<String>,
}

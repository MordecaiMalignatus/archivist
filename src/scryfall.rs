use anyhow::Result;
use anyhow::anyhow;
use reqwest::blocking::Client;

use crate::types::Card;

const SCRYFALL_API_ROOT: &str = "https://api.scryfall.com/";

pub fn query_card(set: &str, number: &str, client: &Client) -> Result<Card> {
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

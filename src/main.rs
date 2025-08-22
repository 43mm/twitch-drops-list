use anyhow::{Context, Result};
use reqwest;
use serde::Deserialize;

const DROPS_API_URL: &str = "https://twitch-drops-api.sunkwi.com/drops";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiGame {
    game_display_name: String,
    #[serde(rename = "rewards")]
    drops: Vec<ApiDrops>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiDrops {
    name: String,
    start_at: String,
    end_at: String,
    #[serde(rename = "timeBasedDrops")]
    rewards: Vec<ApiReward>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiReward {
    name: String,
    #[serde(rename = "requiredMinutesWatched")]
    minutes_required: u16,
}

fn main() -> Result<()> {
    let game_data = fetch_game_data()?;

    Ok(())
}

fn fetch_game_data() -> Result<Vec<ApiGame>> {
    eprintln!("fetching open drop campaigns...");

    let game_data = reqwest::blocking::get(DROPS_API_URL)
        .context("failed to fetch from api")?
        .json::<Vec<ApiGame>>()
        .context("failed to parse json response")?;

    Ok(game_data)
}

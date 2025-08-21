use serde::Deserialize;
use std::error::Error;

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
    required_minutes_watched: u16,
}

fn main() -> Result<(), Box<dyn Error>>{
    println!("fetching open drop campaigns...");

    let json_string  = reqwest::blocking::get("https://twitch-drops-api.sunkwi.com/drops")?.text()?;
    let api_data: Vec<ApiGame> = serde_json::from_str(&json_string)?;

    Ok(())
}

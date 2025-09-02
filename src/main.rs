use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufWriter, Write};

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
    end_at: DateTime<Utc>,
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
    let now = Utc::now();

    let file = File::create("README.md");
    let mut writer = BufWriter::new(file.context("failed to create README.md")?);

    writeln!(writer, "# Twitch Drops Campaigns\n")?;

    for game in game_data {
        writeln!(writer, "{}", game.game_display_name)?;
        for drop in game.drops {
            let days = drop.end_at.signed_duration_since(now).num_days() as i16;
            let end = if days < 0 {
                "already ended".to_string()
            } else {
                format!("ends {}", format_days_from_now(days))
            };
            writeln!(writer, "- {} ({})", drop.name, end)?;
            for reward in drop.rewards {
                writeln!(
                    writer,
                    "  - {} ({} minutes watched)",
                    reward.name, reward.minutes_required
                )?;
            }
            writeln!(writer)?;
        }
    }

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

fn format_days_from_now(days: i16) -> String {
    match days {
        0 => "today".into(),
        1 => "tomorrow".into(),
        _ => format!("in {} days", days),
    }
}

//! Fetch active Twitch Drop campaigns and writes them to README.md
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use tempfile::NamedTempFile;

const DROPS_API_URL: &str = "https://twitch-drops-api.sunkwi.com/drops";
const LATEST_WINDOW_DAYS: i64 = 7;
const FILE_NAME: &str = "DROPS.md";

// Structs for deserialising API response
// ApiGame contains the name of the game and a list of active drop campaigns
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiGame {
    game_display_name: String,
    #[serde(rename = "rewards")]
    drops: Vec<ApiDrops>,
}

// ApiDrops contains the name of the drop campaign, start and end dates, and a list of rewards
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiDrops {
    name: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    #[serde(rename = "timeBasedDrops")]
    rewards: Vec<ApiReward>,
}

// ApiReward contains the name of the reward and the number of minutes watched required to earn it
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiReward {
    name: String,
    #[serde(rename = "requiredMinutesWatched")]
    minutes_required: u16,
}

fn main() -> Result<()> {
    let mut games = fetch_game_data()?;
    games.sort_by_key(|g| g.game_display_name.to_lowercase());

    let mut temp_file = NamedTempFile::new().context("failed to create temporary file")?;

    {
        let mut writer = BufWriter::new(&mut temp_file);
        writeln!(writer, "# Twitch Drops Campaigns\n")?;

        if games.is_empty() {
            writeln!(writer, "No active drops campaigns found.")?;
            return Ok(());
        }

        let now = Utc::now();
        write_latest_drops(&games, now, &mut writer)?;
        write_all_games(&games, now, &mut writer)?;
    }

    temp_file
        .persist(FILE_NAME)
        .context("failed to persist file")?;

    Ok(())
}

// Write the list of drop campaigns that started recently, organised by date
fn write_latest_drops(
    games: &[ApiGame],
    now: DateTime<Utc>,
    writer: &mut impl Write,
) -> Result<()> {
    let updates_from = now - Duration::days(LATEST_WINDOW_DAYS);

    let mut latest_updates: BTreeMap<chrono::NaiveDate, BTreeMap<&str, Vec<&ApiDrops>>> =
        BTreeMap::new();
    for game in games {
        for drop in game.drops.iter().filter(|d| d.start_at > updates_from) {
            latest_updates
                .entry(drop.start_at.date_naive())
                .or_default()
                .entry(&game.game_display_name)
                .or_default()
                .push(drop);
        }
    }

    writeln!(writer, "## Recent Drops\n")?;

    if latest_updates.is_empty() {
        writeln!(
            writer,
            "No drop campaigns started in the last {} days.\n",
            LATEST_WINDOW_DAYS
        )?;
        return Ok(());
    }

    for (date, games_for_date) in latest_updates.iter().rev() {
        writeln!(writer, "{}", date.format("%Y-%m-%d"))?;
        for (game, drops) in games_for_date {
            writeln!(writer, "- {}", escape_markdown(game))?;
            for drop in drops {
                writeln!(
                    writer,
                    "  - {} ({})",
                    escape_markdown(&drop.name),
                    ends_in_days(drop.end_at, now)
                )?;
            }
        }
        writeln!(writer)?;
    }
    Ok(())
}

// Write the full list of currently active drop campaigns by game
fn write_all_games(games: &[ApiGame], now: DateTime<Utc>, writer: &mut impl Write) -> Result<()> {
    writeln!(writer, "## All drops\n")?;
    for game in games {
        writeln!(writer, "{}", escape_markdown(&game.game_display_name))?;
        for drop in &game.drops {
            let end = ends_in_days(drop.end_at, now);
            writeln!(writer, "- {} ({})", escape_markdown(&drop.name), end)?;
            for reward in &drop.rewards {
                writeln!(
                    writer,
                    "  - {} ({} minutes watched)",
                    escape_markdown(&reward.name),
                    reward.minutes_required
                )?;
            }
        }
        writeln!(writer)?;
    }
    Ok(())
}

// Fetches the list of currently active Twitch Drop campaigns, listed by game name
fn fetch_game_data() -> Result<Vec<ApiGame>> {
    eprintln!("fetching open drop campaigns...");

    let game_data = reqwest::blocking::get(DROPS_API_URL)
        .context("failed to fetch from api")?
        .json::<Vec<ApiGame>>()
        .context("failed to parse json response")?;

    Ok(game_data)
}

// Escape markdown special characters
fn escape_markdown(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.'
            | '!' | '|' | '<' | '>' | '~' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

// Calculate days until end date and format as a human-readable string
fn ends_in_days(end: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let days = end.signed_duration_since(now).num_days() as i16;
    if days < 0 {
        return "already ended".to_string();
    }
    return format!("ends {}", format_days_from_now(days));
}

// Format a number of days from now into a human-readable string - for future dates only
fn format_days_from_now(days: i16) -> String {
    match days {
        0 => "today".into(),
        1 => "tomorrow".into(),
        _ => format!("in {} days", days),
    }
}

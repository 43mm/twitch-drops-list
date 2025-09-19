use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};

const DROPS_API_URL: &str = "https://twitch-drops-api.sunkwi.com/drops";
const LATEST_WINDOW_DAYS: i64 = 7;

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
    start_at: DateTime<Utc>,
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
    let mut games = fetch_game_data()?;
    games.sort_by_key(|g| g.game_display_name.to_lowercase());

    let file = File::create("README.md");
    let mut writer = BufWriter::new(file.context("failed to create README.md")?);
    let now = Utc::now();

    writeln!(writer, "# Twitch Drops Campaigns\n")?;
    if games.is_empty() {
        writeln!(writer, "No active drops campaigns found.")?;
        return Ok(());
    }
    write_latest_drops(&games, now, &mut writer)?;
    write_all_games(&games, now, &mut writer)?;

    Ok(())
}

fn write_latest_drops(
    games: &[ApiGame],
    now: DateTime<Utc>,
    writer: &mut impl Write,
) -> Result<()> {
    let updates_from = now - Duration::days(LATEST_WINDOW_DAYS);

    let latest_updates = games
        .iter()
        .flat_map(|game| game.drops.iter().map(move |drop| (game, drop)))
        .filter(|(_, drop)| drop.start_at > updates_from)
        .fold(BTreeMap::new(), |mut acc, (game, drop)| {
            let date = drop.start_at.date_naive();
            let entry = acc.entry(date).or_insert_with(BTreeMap::new);
            entry
                .entry(&game.game_display_name)
                .or_insert_with(Vec::new)
                .push(drop);
            acc
        });

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
                writeln!(writer, "  - {}", escape_markdown(&drop.name))?;
            }
        }
        writeln!(writer)?;
    }
    Ok(())
}

fn write_all_games(games: &[ApiGame], now: DateTime<Utc>, writer: &mut impl Write) -> Result<()> {
    writeln!(writer, "## All drops\n")?;
    for game in games {
        writeln!(writer, "{}", escape_markdown(&game.game_display_name))?;
        for drop in &game.drops {
            let days = drop.end_at.signed_duration_since(now).num_days() as i16;
            let end = if days < 0 {
                "already ended".to_string()
            } else {
                format!("ends {}", format_days_from_now(days))
            };
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

fn fetch_game_data() -> Result<Vec<ApiGame>> {
    eprintln!("fetching open drop campaigns...");

    let game_data = reqwest::blocking::get(DROPS_API_URL)
        .context("failed to fetch from api")?
        .json::<Vec<ApiGame>>()
        .context("failed to parse json response")?;

    Ok(game_data)
}

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

fn format_days_from_now(days: i16) -> String {
    match days {
        0 => "today".into(),
        1 => "tomorrow".into(),
        _ => format!("in {} days", days),
    }
}

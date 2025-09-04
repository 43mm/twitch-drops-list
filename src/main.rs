use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDate, Utc};
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
    games.sort_by(|a, b| {
        a.game_display_name
            .to_lowercase()
            .cmp(&b.game_display_name.to_lowercase())
    });
    let now = Utc::now();

    let file = File::create("README.md");
    let mut writer = BufWriter::new(file.context("failed to create README.md")?);

    writeln!(writer, "# Twitch Drops Campaigns\n")?;
    write_latest_drops(&games, now, &mut writer)?;
    write_all_games(&games, now, &mut writer)?;

    Ok(())
}

fn write_latest_drops(
    games: &Vec<ApiGame>,
    now: DateTime<Utc>,
    writer: &mut BufWriter<File>,
) -> Result<(), anyhow::Error> {
    let updates_from = now - Duration::days(LATEST_WINDOW_DAYS);
    let mut latest_updates: BTreeMap<NaiveDate, BTreeMap<String, Vec<&ApiDrops>>> = BTreeMap::new();
    for game in games {
        for drop in &game.drops {
            if drop.start_at >= updates_from {
                let start_date = drop.start_at.date_naive();
                latest_updates
                    .entry(start_date)
                    .or_default()
                    .entry(game.game_display_name.clone())
                    .or_default()
                    .push(drop);
            }
        }
    }
    let mut dates: Vec<NaiveDate> = latest_updates.keys().cloned().collect();
    dates.sort();
    dates.reverse();
    Ok(if dates.is_empty() {
    } else {
        writeln!(writer, "## Recent Drops\n")?;
        for date in dates {
            writeln!(writer, "{}", date.format("%Y-%m-%d"))?;
            if let Some(games) = latest_updates.get(&date) {
                for (game, drops) in games {
                    writeln!(writer, "- {}", escape_markdown(game))?;
                    for drop in drops {
                        writeln!(writer, "  - {}", escape_markdown(&drop.name))?;
                    }
                }
            }
            writeln!(writer)?;
        }
    })
}

fn write_all_games(
    games: &Vec<ApiGame>,
    now: DateTime<Utc>,
    writer: &mut BufWriter<File>,
) -> Result<(), anyhow::Error> {
    if games.is_empty() {
        writeln!(writer, "No active drops campaigns found.")?;
        return Ok(());
    } else {
        writeln!(writer, "## All drops\n")?;
    }
    Ok(for game in games {
        writeln!(writer, "{}", escape_markdown(&game.game_display_name))?;
        for drop in &game.drops {
            let days = drop.end_at.signed_duration_since(now).num_days() as i16;
            let end = if days < 0 {
                "already ended".to_string()
            } else {
                format!("ends {}", format_days_from_now(days))
            };
            writeln!(writer, "- {} ({})", drop.name, end)?;
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
    })
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
    const ESCAPE_CHARS: &[char] = &[
        '\\', '`', '*', '_', '{', '}', '[', ']', '(', ')', '#', '+', '-', '.', '!', '|', '<', '>',
        '~',
    ];
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if ESCAPE_CHARS.contains(&c) {
            escaped.push('\\');
        }
        escaped.push(c);
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

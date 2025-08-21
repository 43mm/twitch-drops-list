use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>{
    println!("fetching open drop campaigns...");

    let json_string  = reqwest::blocking::get("https://twitch-drops-api.sunkwi.com/drops")?.text()?;
    println!("fetched: {}", &json_string[..500.min(json_string.len())]);

    Ok(())
}

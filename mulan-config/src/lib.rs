use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use self::schema::Config;

mod schema;

fn detect() -> Option<Box<Path>> {
    const POSSIBLE_CONFIG_FILE_LOCATIONS: [&str; 2] =
        [".mulan/mulan.toml", ".config/mulan/mulan.toml"];

    POSSIBLE_CONFIG_FILE_LOCATIONS
        .iter()
        .map(Path::new)
        .find(|path| path.exists())
        .map(Box::from)
}

pub fn parse() -> Result<Config> {
    let path = detect().context("no config file found")?;
    let raw = fs::read_to_string(path)?;
    let parsed = toml::from_str(&raw)?;
    Ok(parsed)
}

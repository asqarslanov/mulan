use std::path::Path;

use anyhow::Result;

mod input;
mod output;

pub fn parse_translation(name: &str) -> Result<output::Translation> {
    let path = Path::new("locales").join(name).join("locale.json5");
    let raw = std::fs::read_to_string(path)?;
    let parsed: input::Translation = json5::from_str(&raw)?;
    Ok(parsed.into())
}

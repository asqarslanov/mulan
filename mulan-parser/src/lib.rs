use std::path::Path;

use anyhow::Result;
use mulan_config::Config;

mod input;
mod output;

pub fn parse_translation(name: &str, config: &Config) -> Result<output::Translation> {
    let entry_point = Path::new(&*config.general.entry_point)
        .with_extension(<&str>::from(&config.general.locale_extension));

    let path = config.general.locales_dir.join(name).join(entry_point);
    let parsed = load_file(path)?;
    parsed.try_into()
}

pub(crate) fn load_file(path: impl AsRef<Path>) -> Result<input::Translation> {
    let raw = std::fs::read_to_string(path)?;
    let parsed = json5::from_str(&raw)?;
    Ok(parsed)
}

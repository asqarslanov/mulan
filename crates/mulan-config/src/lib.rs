//! See [`Config`].

use std::collections::BTreeSet;

use serde_with::{SetPreventDuplicates, serde_as};
use serdev::Deserialize;

pub use self::language::Language;

mod language;

/// ...
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(validate = "Self::validate")]
struct Config {
    /// ...
    #[serde_as(as = "SetPreventDuplicates<_>")]
    pub locales: BTreeSet<Language>,

    /// ...
    pub default_locale: Language,
}

impl Config {
    fn validate(&self) -> Result<(), String> {
        if !self.locales.contains(&self.default_locale) {
            return Err(format!(
                "`locales` should contain default locale (\"{}\")",
                self.default_locale.tag(),
            ));
        }
        Ok(())
    }
}

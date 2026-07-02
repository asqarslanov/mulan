//! See [`Config`].

use std::collections::BTreeSet;

use serde_with::{SetPreventDuplicates, serde_as};
use serdev::Deserialize;

/// A [BCP 47 language tag](https://en.wikipedia.org/wiki/IETF_language_tag)
/// used as a locale name (e.g., `en-US` or `ru-RU`).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum Language {
    /// English (United States)
    #[serde(rename = "en-US")]
    EnUs,
}

impl Language {
    fn tag(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
        }
    }
}

/// ...
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(validate = "Self::validate")]
struct Input {
    /// ...
    #[serde_as(as = "SetPreventDuplicates<_>")]
    pub locales: BTreeSet<Language>,

    /// ...
    pub default_locale: Language,
}

impl Input {
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

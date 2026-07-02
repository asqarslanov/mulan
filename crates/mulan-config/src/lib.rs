//! See [`Config`].

use std::collections::BTreeSet;

use serde::Deserialize;
use serde_with::{SetPreventDuplicates, serde_as};

/// A [BCP 47 language tag](https://en.wikipedia.org/wiki/IETF_language_tag)
/// used as a locale name (e.g., `en-US` or `ru-RU`).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum Language {
    /// English (United States)
    #[serde(rename = "en-US")]
    EnUs,
}

/// ...
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Input {
    /// ...
    #[serde_as(as = "SetPreventDuplicates<_>")]
    pub locales: BTreeSet<Language>,

    /// ...
    pub default_locale: Language,
}

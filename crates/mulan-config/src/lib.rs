//! See [`Config`].

use mitsein::btree_set1::BTreeSet1;

/// A [BCP 47 language tag](https://en.wikipedia.org/wiki/IETF_language_tag)
/// used as a locale name (e.g., `en-US` or `ru-RU`).
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Language {
    /// English (United States)
    EnUs,
}

/// ...
#[derive(Debug)]
pub struct Config {
    /// ...
    pub default_locale: Language,

    /// ...
    pub locales: BTreeSet1<Language>,
}

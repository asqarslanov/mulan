use mitsein::str1;
use mitsein::str1::Str1;
use serde::Deserialize;

/// A unique identifier of a human language
/// (e.g., English, Canadian French, or Esperanto).
///
/// Uses [`Self::tag`] for de/serialization.
#[expect(
    clippy::unsafe_derive_deserialize,
    reason = "can only be deserialized from a hard-coded subset of strings"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum Language {
    /// English (United States)
    #[serde(rename = "en-US")]
    EnUs,

    /// French (Canada)
    #[serde(rename = "fr-CA")]
    FrCa,

    /// Russian (Russia)
    #[serde(rename = "ru-RU")]
    RuRu,
}

impl Language {
    /// Returns the corresponding
    /// [BCP 47 language tag](https://en.wikipedia.org/wiki/IETF_language_tag)
    /// (e.g., `en`, `fr-CA`, or `eo`).
    #[must_use]
    pub const fn tag(&self) -> &'static Str1 {
        match self {
            Self::EnUs => str1!("en-US"),
            Self::FrCa => str1!("fr-CA"),
            Self::RuRu => str1!("ru-RU"),
        }
    }
}

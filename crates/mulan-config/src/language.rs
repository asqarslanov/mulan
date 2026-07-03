use serde::Deserialize;

/// A unique identifier of a human language
/// (e.g., English, Canadian French, or Esperanto).
///
/// Uses [`Self::tag`] for de/serialization.
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
    pub const fn tag(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::FrCa => "fr-CA",
            Self::RuRu => "ru-RU",
        }
    }
}

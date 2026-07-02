use serde::Deserialize;

/// A [BCP 47 language tag](https://en.wikipedia.org/wiki/IETF_language_tag)
/// used as a locale name (e.g., `en-US` or `ru-RU`).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum Language {
    /// English (United States)
    #[serde(rename = "en-US")]
    EnUs,
}

impl Language {
    /// ...
    pub fn tag(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
        }
    }
}

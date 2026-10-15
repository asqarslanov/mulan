use mitsein::str1;
use mitsein::str1::Str1;
use serde::{Deserialize, Serialize};

/// A unique identifier of a human language
/// (e.g., English, Canadian French, or Esperanto).
///
/// Uses [`Self::tag`] for de/serialization.
#[expect(
    clippy::unsafe_derive_deserialize,
    reason = "can only be deserialized from a hard-coded subset of strings"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    #[serde(rename = "ru")]
    Ru,

    #[serde(rename = "en")]
    En,

    #[serde(rename = "en-GB")]
    EnGb,

    #[serde(rename = "eo")]
    Eo,

    #[serde(rename = "tt")]
    Tt,

    #[serde(rename = "fr")]
    Fr,

    #[serde(rename = "es")]
    Es,

    #[serde(rename = "es-US")]
    EsUs,

    #[serde(rename = "es-ES")]
    EsEs,

    #[serde(rename = "es-MX")]
    EsMx,

    #[serde(rename = "es-419")]
    Es419,
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
            Self::Ru => str1!("ru"),
            Self::En => str1!("en"),
            Self::EnGb => str1!("en-GB"),
            Self::Eo => str1!("eo"),
            Self::Tt => str1!("tt"),
            Self::Fr => str1!("fr"),
            Self::Es => str1!("es"),
            Self::EsUs => str1!("es-US"),
            Self::EsEs => str1!("es-ES"),
            Self::EsMx => str1!("es-MX"),
            Self::Es419 => str1!("es-419"),
        }
    }

    /// Returns the full language name with an optional subtag.
    ///
    /// For example:
    /// - `English (United States)`
    /// - `French (Canada)`
    /// - `Esperanto`
    #[must_use]
    pub const fn name(&self) -> &'static Str1 {
        match self {
            Self::EnUs => str1!("English (United States)"),
            Self::FrCa => str1!("French (Canada)"),
            Self::RuRu => str1!("Russian (Russia)"),
            Self::Ru => str1!("Russian"),
            Self::En => str1!("English"),
            Self::EnGb => str1!("English (United Kingdom)"),
            Self::Eo => str1!("Esperanto"),
            Self::Tt => str1!("Tatar"),
            Self::Fr => str1!("French"),
            Self::Es => str1!("Spanish"),
            Self::EsEs => str1!("Spanish (Spain)"),
            Self::EsMx => str1!("Spanish (Mexico)"),
            Self::EsUs => str1!("Spanish (United States)"),
            Self::Es419 => str1!("Spanish (Latin America)"),
        }
    }

    /// Similar to [`Self::tag`], but the result is in `PascalCase`
    /// (e.g., `EnUs`, `FrCa`, or `Eo`).
    #[must_use]
    pub const fn tag_pascal_case(&self) -> &'static Str1 {
        match self {
            Self::EnUs => str1!("EnUs"),
            Self::FrCa => str1!("FrCa"),
            Self::RuRu => str1!("RuRu"),
            Self::Ru => str1!("Ru"),
            Self::En => str1!("En"),
            Self::EnGb => str1!("EnGb"),
            Self::Eo => str1!("Eo"),
            Self::Tt => str1!("Tt"),
            Self::Fr => str1!("Fr"),
            Self::Es => str1!("Es"),
            Self::EsEs => str1!("EsEs"),
            Self::EsMx => str1!("EsMx"),
            Self::EsUs => str1!("EsUs"),
            Self::Es419 => str1!("Es419"),
        }
    }
}

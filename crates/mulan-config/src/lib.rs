//! See [`Config`].

use std::collections::BTreeSet;

use serde_with::{SetPreventDuplicates, serde_as};

pub use self::language::Language;

mod language;

/// ...
#[serde_as]
#[derive(Debug, PartialEq, Eq, serdev::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(validate = "Self::validate")]
struct Config {
    /// ...
    #[serde_as(as = "SetPreventDuplicates<_>")]
    pub locales: BTreeSet<Language>,

    /// ...
    pub default_locale: Language,
}

enum ParseConfigError {}

impl Config {
    /// ...
    pub fn parse() -> Result<Self, ParseConfigError> {
        todo!();
    }

    /// ...
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("", None)]
    #[case(
        indoc! {r#"
            locales = ["en-US"]
            default-locale = "en-US"
        "#},
        Some(Config {
            locales: [Language::EnUs].iter().copied().collect(),
            default_locale: Language::EnUs,
        }),
    )]
    #[case(
        indoc! {r#"
            locales = []
            default-locale = "fr-CA"
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            default-locale = "ru-RU"
            locales = ["ru-RU", "en-US"]
        "#},
        Some(Config {
            locales: [Language::EnUs, Language::RuRu].iter().copied().collect(),
            default_locale: Language::RuRu,
        }),
    )]
    #[case(
        indoc! {r#"
            default-locale = "ru-RU"
            locales = ["ru-RU", "en-US", "ru-RU"]
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            default-locale = "ru-RU"
            locales = ["fr-CA", "en-US"]
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            locales = ["xx-XX"]
            default-locale = "xx-XX"
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            locales = ["ru-RU", "xx-XX"]
            default-locale = "ru-RU"
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            locales = ["ru-RU", "en-us"]
            default-locale = "ru-RU"
        "#},
        None,
    )]
    fn parse(#[case] input: &str, #[case] expected_output: Option<Config>) {
        let actual_output: Option<Config> = toml::from_str(input).ok();
        assert_eq!(actual_output, expected_output);
    }
}

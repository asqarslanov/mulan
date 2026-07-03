//! See [`Config`].

use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

use figment2::Figment;
use figment2::providers::{Format as _, Toml};
use serde_with::{SetPreventDuplicates, serde_as};

pub use self::language::Language;

mod language;

/// ...
#[serde_as]
#[derive(Debug, PartialEq, Eq, serdev::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(validate = "Self::validate")]
pub struct Config {
    /// ...
    #[serde_as(as = "SetPreventDuplicates<_>")]
    pub locales: BTreeSet<Language>,

    /// ...
    pub default_locale: Language,
}

/// Errors of [`Config::locate_and_read`].
#[derive(Debug)]
pub enum ReadConfigError {
    /// Failed to read the file.
    Io { path: PathBuf, error: io::Error },

    /// Failed to parse the TOML file according to the schema.
    Format(toml::de::Error),
}

impl Config {
    /// ...
    pub fn locate_and_read() -> figment2::Result<Self> {
        Figment::new().merge(Toml::file("mulan.toml")).extract()
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
    use indoc::{formatdoc, indoc};
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
        figment2::Jail::expect_with(|jail| {
            jail.create_file("mulan.toml", input)?;
            let actual_output = Config::locate_and_read().ok();
            if actual_output == expected_output {
                Ok(())
            } else {
                Err(formatdoc! {"
                    assertion `left == right` failed
                      left: {actual_output:?}
                     right: {expected_output:?}\
                "}
                .into())
            }
        });
    }
}

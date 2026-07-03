//! See [`Config`].

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

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

/// Errors of [`Config::locate_and_read`].
enum ReadConfigError {
    /// Failed to read the file.
    Io { path: PathBuf, error: io::Error },

    /// Failed to parse the TOML file according to the schema.
    Format(toml::de::Error),
}

impl Config {
    /// ...
    pub fn locate_and_read() -> Result<Self, ReadConfigError> {
        Self::read(Path::new("mulan.toml").into())
    }

    /// ...
    fn read(path: Cow<'_, Path>) -> Result<Self, ReadConfigError> {
        let file_contents = fs::read_to_string(&path).map_err(|error| ReadConfigError::Io {
            path: path.into_owned(),
            error,
        })?;
        toml::from_str(&file_contents).map_err(ReadConfigError::Format)
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
    use std::io::Write as _;

    use indoc::indoc;
    use rstest::rstest;
    use tempfile::NamedTempFile;

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
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{input}").unwrap();
        let actual_output = Config::read(file.path().into()).ok();
        assert_eq!(actual_output, expected_output);
    }
}

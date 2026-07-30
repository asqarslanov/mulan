//! # Mulan Config
//!
//! This crate defines the user config structure so that the user
//! can choose specific behaviors of various parts of the system
//! (e.g., locale parsing or binding generation).
//!
//! This crate is responsible for locating the user config, parsing and
//! validating its contents, and obtaining runtime-specific metadata.
//!
//! See [`Config`] and [`Config::locate_and_read`].

use std::collections::BTreeSet;

use figment2::Figment;
use figment2::providers::{Format as _, Toml};
use serde_with::{SetPreventDuplicates, serde_as};

pub use self::language::Language;
pub use self::meta::{ConfigMeta, MetaError};

mod language;
mod meta;

/// # Mulan Config
///
/// This file is used to configure Mulan project-wide.
///
/// Mulan is an i18n framework.
/// See <https://github.com/asqarslanov/mulan> for more details.
#[serde_as]
#[derive(Debug, PartialEq, Eq, serdev::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(validate = "Self::validate")]
pub struct Config {
    /// Information about the execution context obtained at runtime.
    #[serde(skip)]
    pub meta: ConfigMeta,

    /// All languages you want to translate your app into.
    ///
    /// A list of non-duplicate
    /// [BCP 47 language tags](https://en.wikipedia.org/wiki/IETF_language_tag).
    ///
    /// Must include `default-locale`.
    #[serde_as(as = "SetPreventDuplicates<_>")]
    pub locales: BTreeSet<Language>,

    /// The main locale of your application.
    ///
    /// All other locales have to conform to its schema.
    ///
    /// Acts as a fallback locale if a translation does not exist
    /// in another locale.
    pub default_locale: Language,
}

impl crate::Config {
    /// Tries to find the most appropriate config file in the filesystem
    /// and read + validate it.
    ///
    /// Uses [`mod@figment2`] under the hood.
    #[allow(clippy::result_large_err)]
    pub fn locate_and_read() -> Result<Self, ConfigError> {
        let figment = Figment::from(Toml::file("mulan.toml"));
        let meta = ConfigMeta::compute(&figment).map_err(ConfigError::Meta)?;
        let mut config = figment.extract::<Self>().map_err(ConfigError::Figment);
        if let Ok(config) = &mut config {
            // Its value was `serde(skip)`ped (only available at runtime).
            config.meta = meta;
        }
        config
    }

    /// Returns an iterator over [`Self::locales`]
    /// with [`Self::default_locale`] filtered out.
    pub fn locales_except_default(&self) -> impl Iterator<Item = Language> {
        self.locales
            .iter()
            .copied()
            .filter(|&locale| locale != self.default_locale)
    }

    /// Used at deserialization with [`mod@serdev`].
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

/// Errors of [`Config::locate_and_read`].
#[derive(Debug)]
pub enum ConfigError {
    /// An error of the underlying library that handles parsing the config.
    Figment(figment2::Error),

    /// An error while obtaining runtime context.
    Meta(MetaError),
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
        Some(crate::Config {
            meta: ConfigMeta::default(),
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
        Some(crate::Config {
            meta: ConfigMeta::default(),
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
    fn read(#[case] input: &str, #[case] expected_output: Option<crate::Config>) {
        figment2::Jail::expect_with(|jail| {
            jail.create_file("mulan.toml", input)?;
            let mut actual_output = crate::Config::locate_and_read().ok();
            if let Some(config) = &mut actual_output {
                config.meta = ConfigMeta::default();
            }
            if actual_output != expected_output {
                return Err(formatdoc! {"
                    assertion `left == right` failed
                      left: {actual_output:?}
                     right: {expected_output:?}\
                "}
                .into());
            }
            Ok(())
        });
    }
}

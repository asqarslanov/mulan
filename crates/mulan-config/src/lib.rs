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
use mitsein::small_vec1::SmallVec1;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use serde_with::{SetPreventDuplicates, serde_as};

pub use self::language::Language;
pub use self::meta::ConfigMeta;
use crate::errors::{ConfigError, FigmentError, LocateError, LocateIoError, NotFoundError};

pub mod errors;
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
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
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
    /// Must include `main-locale`.
    #[serde_as(as = "SetPreventDuplicates<_>")]
    pub locales: BTreeSet<Language>,

    /// The primary locale of your application.
    ///
    /// All other locales have to conform to its schema.
    ///
    /// Acts as a fallback locale if a translation does not exist
    /// in another locale.
    pub main_locale: Language,

    /// The list of targets (i.e., programming languages) for which
    /// i18n bindings should be generated.
    pub generate: Option<SmallVec1<[Target; 1]>>,

    /// Your preferred convention to name keys in locale definitions.
    ///
    /// The default value is `"kebab-case"`.
    #[serde(skip)]
    pub key_case: Case,
}

/// See [`crate::Config::generate`].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "target")]
pub enum Target {
    /// The Rust programming language.
    Rust(RustTarget),
}

/// See [`Target::Rust`].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTarget {
    /// Where to generate a Rust module with i18n bindings.
    pub file: RelativePathBuf,
}

/// Word case (e.g., `camelCase`, `kebab-case`, or `snake_case`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Case {
    /// Lowercase dash-separated words (`kebab-case`).
    #[default]
    #[serde(rename = "kebab-case")]
    Kebab,

    /// Words, each starting with a capital letter, without separators
    /// in-between (`PascalCase`).
    #[serde(rename = "PascalCase")]
    Pascal,

    /// Lowercase underscore-separated words (`snake_case`).
    #[serde(rename = "snake_case")]
    Snake,
}

impl crate::Config {
    /// Tries to find the most appropriate config file in the filesystem
    /// and read + validate it.
    ///
    /// Uses [`mod@figment2`] under the hood.
    #[expect(
        clippy::result_large_err,
        reason = "`ConfigError` is mostly a wrapper around `figment2::Error`"
    )]
    pub fn locate_and_read() -> Result<Self, ConfigError> {
        let figment = Figment::from(Toml::file("mulan.toml"));
        let meta = ConfigMeta::compute(&figment).map_err(ConfigError::Meta)?;
        let mut config = {
            figment
                .extract::<Self>()
                .map_err(|inner| ConfigError::Figment(FigmentError { inner }))
        };
        if let Ok(config) = &mut config {
            // Its value was `serde(skip)`ped (only available at runtime).
            config.meta = meta;
        }
        config
    }

    ///
    pub fn locate_without_parents() -> Result<RelativePathBuf, LocateError> {
        let path = RelativePathBuf::from("mulan.toml");
        let exists = match path.to_path("").try_exists() {
            Ok(exists) => exists,
            Err(error) => return Err(LocateError::Io(LocateIoError { path, error })),
        };
        if exists {
            Ok(path)
        } else {
            Err(LocateError::NotFound(NotFoundError))
        }
    }

    /// Returns an iterator over [`Self::locales`]
    /// with [`Self::main_locale`] filtered out.
    pub fn locales_except_main(&self) -> impl Iterator<Item = Language> {
        self.locales
            .iter()
            .copied()
            .filter(|&locale| locale != self.main_locale)
    }

    /// A basic stand-in for a config without useful data.
    ///
    /// Can be used when an instance of [`crate::Config`] is required, but none
    /// is available.
    #[must_use]
    pub fn dummy() -> Self {
        Self {
            meta: ConfigMeta::default(),
            locales: BTreeSet::default(),
            main_locale: Language::EnUs,
            generate: None,
            key_case: Case::Kebab,
        }
    }

    /// Used at deserialization with [`mod@serdev`].
    fn validate(&self) -> Result<(), String> {
        if !self.locales.contains(&self.main_locale) {
            return Err(format!(
                "`locales` should contain main locale (\"{}\")",
                self.main_locale.tag(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use indoc::{formatdoc, indoc};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("", None)]
    #[case(
        indoc! {r#"
            locales = ["en-US"]
            main-locale = "en-US"
        "#},
        Some(crate::Config {
            meta: ConfigMeta::default(),
            locales: iter::once(Language::EnUs).collect(),
            main_locale: Language::EnUs,
            generate: None,
            key_case: Case::Kebab,
        }),
    )]
    #[case(
        indoc! {r#"
            locales = []
            main-locale = "fr-CA"
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            main-locale = "ru-RU"
            locales = ["ru-RU", "en-US"]
        "#},
        Some(crate::Config {
            meta: ConfigMeta::default(),
            locales: [Language::EnUs, Language::RuRu].iter().copied().collect(),
            main_locale: Language::RuRu,
            generate: None,
            key_case: Case::Kebab,
        }),
    )]
    #[case(
        indoc! {r#"
            main-locale = "ru-RU"
            locales = ["ru-RU", "en-US", "ru-RU"]
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            main-locale = "ru-RU"
            locales = ["fr-CA", "en-US"]
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            locales = ["xx-XX"]
            main-locale = "xx-XX"
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            locales = ["ru-RU", "xx-XX"]
            main-locale = "ru-RU"
        "#},
        None,
    )]
    #[case(
        indoc! {r#"
            locales = ["ru-RU", "en-us"]
            main-locale = "ru-RU"
        "#},
        None,
    )]
    fn read(#[case] input: &str, #[case] expected_output: Option<crate::Config>) {
        figment2::Jail::expect_with(
            #[expect(clippy::result_large_err, reason = "done by the book")]
            |jail| {
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
            },
        );
    }
}

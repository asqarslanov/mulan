//! See [`Config`] and [`Config::locate_and_read`].

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::{env, io};

use figment2::Figment;
use figment2::providers::{Format as _, Toml};
use itertools::Itertools as _;
use mitsein::btree_set1::BTreeSet1;
use mitsein::iter1::IteratorExt as _;
use relative_path::RelativePathBuf;
use serde_with::{SetPreventDuplicates, serde_as};

pub use self::language::Language;

mod language;

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
        let meta = ConfigMeta::new(&figment).map_err(ConfigError::Meta)?;
        let mut config = figment.extract::<Self>().map_err(ConfigError::Figment);
        if let Ok(config) = &mut config {
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

    /// ...
    Meta(MetaError),
}

/// See [`Config::meta`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConfigMeta {
    /// See [`std::env::current_dir`].
    pub current_dir: PathBuf,

    /// The path of the project root directory Mulan is operating on,
    /// relative to [`Self::current_dir`].
    pub root_dir: RelativePathBuf,
}

/// ...
#[derive(Debug)]
pub enum MetaError {
    /// Failed to call [`std::env::current_dir`].
    CurrentDir(io::Error),

    /// Unable to locate a config file anywhere.
    SourceNotFound,

    /// Multiple config files found (only one is permitted).
    AmbiguousSource(BTreeSet1<RelativePathBuf>),
}

impl ConfigMeta {
    fn new(figment: &Figment) -> Result<Self, MetaError> {
        let current_dir = env::current_dir().map_err(MetaError::CurrentDir)?;
        let (root_dir, _config_file) = {
            figment
                .metadata()
                .filter_map(|metadata| {
                    let source_absolute = {
                        metadata
                            .source
                            .as_ref()
                            .expect("all sources are predetermined")
                            .file_path()
                            .expect("config is only read from a file")
                    };
                    if source_absolute.is_relative() {
                        return None;
                    }
                    let (root_dir_absolute, config_file) = {
                        source_absolute
                            .parent()
                            .zip(source_absolute.file_name().and_then(OsStr::to_str))
                            .expect("config source should point to a file")
                    };
                    let root_dir_raw = pathdiff::diff_paths(root_dir_absolute, &current_dir)
                        .expect("current_dir can be subtracted from config source");
                    let root_dir = RelativePathBuf::from_path(root_dir_raw)
                        .expect("pathdiff::diff_paths returns a relative path");
                    Some((root_dir, config_file))
                })
                .exactly_one()
                .map_err(|locations| {
                    locations
                        .try_into_iter1()
                        .map_or(MetaError::SourceNotFound, |sources_raw| {
                            let sources = {
                                sources_raw
                                    .map(|(root_dir, config_file)| root_dir.join(config_file))
                                    .collect1()
                            };
                            MetaError::AmbiguousSource(sources)
                        })
                })?
        };
        Ok(Self {
            current_dir,
            root_dir,
        })
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

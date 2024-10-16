use std::path::Path;

use serde::Deserialize;

use super::{locale, target};

mod default;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocaleExtension {
    #[default]
    Json5,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "default::locales_dir")]
    pub locales_dir: Box<Path>,
    #[serde(default)]
    pub main_locale: locale::Name,
    #[serde(default = "default::entry_point")]
    pub entry_point: Box<str>,
    #[serde(default)]
    pub locale_extension: LocaleExtension,
    #[serde(default)]
    pub locales: Box<[locale::Name]>,
    #[serde(default)]
    pub integrations: Box<[target::Name]>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            locales_dir: default::locales_dir(),
            main_locale: locale::Name::default(),
            entry_point: default::entry_point(),
            locale_extension: LocaleExtension::default(),
            locales: Box::from([]),
            integrations: Box::from([]),
        }
    }
}

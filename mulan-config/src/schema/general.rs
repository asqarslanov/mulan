use std::path::Path;

use serde::Deserialize;

use super::{locale, target};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub locales_dir: Box<Path>,
    pub main_locale: locale::Name,
    pub locales: Box<[locale::Name]>,
    pub integrations: Box<[target::Name]>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            locales_dir: Box::from(Path::new("locales/")),
            main_locale: locale::Name::default(),
            locales: Box::default(),
            integrations: Box::default(),
        }
    }
}

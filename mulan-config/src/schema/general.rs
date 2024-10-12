use std::path::Path;

use serde::Deserialize;

use super::{integration, locale};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    locales_dir: Box<Path>,
    main_locale: locale::Name,
    locales: Box<[locale::Name]>,
    integrations: Box<[integration::Name]>,
}

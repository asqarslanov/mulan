use std::collections::HashMap;

use serde::Deserialize;

mod general;
mod integration;
mod locale;

#[derive(Debug, Deserialize)]
pub struct Config {
    general: general::Config,
    locale: HashMap<locale::Name, locale::Config>,
    integration: HashMap<integration::Name, integration::Config>,
}

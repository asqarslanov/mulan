use std::collections::HashMap;

use serde::Deserialize;

mod general;
mod locale;
mod target;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    general: general::Config,
    #[serde(default)]
    locale: HashMap<locale::Name, locale::Config>,
    #[serde(default)]
    target: HashMap<target::Name, target::Config>,
}

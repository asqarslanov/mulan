use std::collections::HashMap;

use serde::Deserialize;

mod general;
mod locale;
mod target;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: general::Config,
    #[serde(default)]
    pub locale: HashMap<locale::Name, locale::Config>,
    #[serde(default)]
    pub target: HashMap<target::Name, target::Config>,
}

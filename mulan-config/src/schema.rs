use std::collections::HashMap;

use serde::Deserialize;

mod general;
mod locale;
mod target;

#[derive(Debug, Deserialize)]
pub struct Config {
    general: general::Config,
    locale: HashMap<locale::Name, locale::Config>,
    target: HashMap<target::Name, target::Config>,
}

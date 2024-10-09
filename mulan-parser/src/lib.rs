use std::{collections::HashMap, path::Path};

use anyhow::Result;

mod types;

pub type Translation = HashMap<String, String>;

pub fn parse_translation(name: &str) -> Result<Translation> {
    read_file(Path::new("locales").join(name).join("locale.json5"));
    todo!()
}

fn read_file(path: impl AsRef<Path>) -> String {
    todo!()
}

use std::{collections::HashMap, path::Path};

use serde::Deserialize;

mod validate;

pub type Name = Box<str>;
pub type Translation = HashMap<Name, Rhs>;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Rhs {
    Text(Box<str>),
    Expanded(Expanded),
    Bool(bool),
    #[serde(deserialize_with = "validate::tag_import")]
    Import(ImportProps),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ImportProps {
    #[serde(deserialize_with = "validate::only_true")]
    True,
    Path(Box<Path>),
}

#[derive(Deserialize)]
pub struct Expanded {
    #[serde(alias = "txt")]
    pub text: Box<str>,
}

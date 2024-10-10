use std::collections::HashMap;

use serde::Deserialize;

pub type Name = Box<str>;
pub type Translation = HashMap<Name, Rhs>;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Rhs {
    Text(Box<str>),
    Expanded(Expanded),
    Bool(bool),
    Import(Import, ImportProps),
}

#[derive(Deserialize)]
#[serde(rename = "import")]
pub struct Import;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ImportProps {
    Bool(bool),
}

#[derive(Deserialize)]
pub struct Expanded {
    #[serde(alias = "txt")]
    pub text: Box<str>,
}

use std::collections::HashMap;

use serde::Deserialize;

pub type Translation = HashMap<Name, Rhs>;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Rhs {
    String(Box<str>),
    Bool(bool),
}

pub type Name = Box<str>;

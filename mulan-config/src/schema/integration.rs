use serde::Deserialize;

pub type Name = Box<str>;

#[derive(Debug, Deserialize)]
pub struct Config {}

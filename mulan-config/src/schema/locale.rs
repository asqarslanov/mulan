use serde::Deserialize;

pub type Name = Box<str>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub decimal_separator: Box<str>,
    pub grouping_separator: Box<str>,
    pub date_format: Box<str>,
}

use serde::Deserialize;

pub type Name = Box<str>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    decimal_separator: Box<str>,
    grouping_separator: Box<str>,
    date_format: Box<str>,
}

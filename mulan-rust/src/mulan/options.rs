use strum::Display;

pub use self::date::DateFormat;

mod date;

#[derive(Debug, Display)]
pub enum DecimalSeparator {
    #[strum(to_string = ".")]
    Dot,
    #[strum(to_string = ",")]
    Comma,
    #[strum(to_string = "{0}")]
    Other(char),
}

#[derive(Debug, Display)]
pub enum GroupingSeparator {
    #[strum(to_string = ".")]
    Dot,
    #[strum(to_string = ",")]
    Comma,
    #[strum(to_string = " ")]
    Space,
    #[strum(to_string = "{0}")]
    Other(char),
}

#[derive(Debug, Default)]
pub struct Options {
    pub decimal_separator: Option<DecimalSeparator>,
    pub grouping_separator: Option<GroupingSeparator>,
    pub date_foramt: Option<DateFormat>,
}

//! Defines structures this crate operates on and operations on them.
//!
//! Most notably, [`Input`] and [`Output`].

use std::fs;
use std::io;
use std::iter;
use std::path::PathBuf;

use mulan_config::Language;

use self::input::{Definition, Input};

mod input;

#[derive(Debug)]
enum ReadError {
    Io { path: PathBuf, error: io::Error },
    Format(serde_saphyr::Error),
}

impl Input {
    fn read() -> Result<Self, ReadError> {
        let en_us_path = PathBuf::from("locales/en-US/locale.yaml");
        let en_us_definition = Definition::read(en_us_path)?;
        let locales = iter::once((Language::EnUs, en_us_definition)).collect();
        Ok(Input { locales })
    }
}

impl Definition {
    fn read(path: PathBuf) -> Result<Self, ReadError> {
        let file_contents =
            fs::read_to_string(&path).map_err(|error| ReadError::Io { error, path })?;
        serde_saphyr::from_str(&file_contents).map_err(ReadError::Format)
    }
}

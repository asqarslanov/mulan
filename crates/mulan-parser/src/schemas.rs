//! Defines structures this crate operates on and operations on them.
//!
//! Most notably, [`Input`] and [`Output`].

use std::fs;
use std::iter;
use std::path::PathBuf;

use mulan_config::Language;

use self::input::{Definition, Input};

mod input;

#[derive(Debug)]
enum ReadError {
    Fs(PathBuf),
    Yaml(serde_saphyr::Error),
}

impl Input {
    fn read() -> Result<Self, ReadError> {
        let file_path = PathBuf::from("locales/en-US/locale.yaml");
        let file_contents = fs::read_to_string(&file_path).map_err(|_| ReadError::Fs(file_path))?;
        let locale_definition: Definition =
            serde_saphyr::from_str(&file_contents).map_err(ReadError::Yaml)?;
        let result = Input {
            locales: iter::once((Language::EnUs, locale_definition)).collect(),
        };
        Ok(result)
    }
}

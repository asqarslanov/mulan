//! Defines structures this crate operates on and operations on them.
//!
//! Most notably, [`Input`] and [`Output`].

use std::fs::File;
use std::io::BufReader;
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
        let path = PathBuf::from("locales/en-US/locale.yaml");
        let file = File::open(&path).map_err(|_| ReadError::Fs(path))?;
        let en_us: Definition =
            serde_saphyr::from_reader(BufReader::new(file)).map_err(ReadError::Yaml)?;
        let result = Input {
            locales: iter::once((Language::EnUs, en_us)).collect(),
        };
        Ok(result)
    }
}

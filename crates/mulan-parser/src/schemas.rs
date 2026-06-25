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

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use compact_str::CompactString;
    use indoc::indoc;
    use rstest::rstest;
    use tempfile::NamedTempFile;

    use super::input::{RawNamespace, RawNode};
    use super::*;

    #[rstest]
    #[case(<&str>::default(), iter::empty())]
    #[case(
        indoc! {r#"
            foo: "Hello"
        "#},
        [(
            CompactString::new("foo"),
            RawNode::Message(CompactString::new("Hello")),
        )]
    )]
    fn read_definition(
        #[case] input: &str,
        #[case] expected_output: impl IntoIterator<Item = (CompactString, RawNode)>,
    ) {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{input}").unwrap();
        let actual_output = Definition::read(file.path().to_owned()).unwrap();
        let expected_output = Definition {
            root: RawNamespace {
                map: expected_output.into_iter().collect(),
            },
        };
        assert_eq!(actual_output, expected_output);
    }
}

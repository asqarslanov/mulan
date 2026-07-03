//! Defines structures this crate operates on and operations on them.
//!
//! Most notably, [`Input`] and [`Output`].

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::{fs, io, iter};

use mulan_config::Language;

use self::input::Definition;
pub use self::input::Input; // TODO: use it privately

mod input;
pub mod output;

/// Errors of [`Input::read`].
#[derive(Debug)]
pub enum ReadError {
    /// Failed to read a file.
    Io { path: PathBuf, error: io::Error },

    /// Failed to parse a YAML file according to the schema.
    Format(serde_saphyr::Error),
}

impl Input {
    /// Locates and parses YAML locale definition files to Rust values.
    pub fn read() -> Result<Self, ReadError> {
        let en_us_path = Path::new("locales/en-US.yaml");
        let en_us_definition = Definition::read(en_us_path.into())?;
        let locales = iter::once((Language::EnUs, en_us_definition)).collect();
        Ok(Self { locales })
    }
}

impl Definition {
    /// Parses a YAML locale definition file to a Rust value.
    fn read(path: Cow<'_, Path>) -> Result<Self, ReadError> {
        let file_contents = fs::read_to_string(&path).map_err(|error| ReadError::Io {
            error,
            path: path.into_owned(),
        })?;
        serde_saphyr::from_str(&file_contents).map_err(ReadError::Format)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use compact_str::CompactString;
    use foldhash::HashMap;
    use indoc::indoc;
    use rstest::rstest;
    use tempfile::NamedTempFile;

    use super::input::{RawNamespace, RawNode};
    use super::*;

    #[rstest]
    #[case(<&str>::default(), Some(iter::empty()))]
    #[case(
        indoc! {r#"
            foo: "Hello"
            bar: "Hi"
        "#},
        Some([
            ("foo".into(), RawNode::Message("Hello".into())),
            ("bar".into(), RawNode::Message("Hi".into())),
        ]),
    )]
    #[case(
        indoc! {r#"
            foo: "Hello"
            foo: "Hi"
        "#},
        None::<[_; 0]>,
    )]
    #[case(
        indoc! {r#"
            namespace:
              foo: "Hello"
              foo: "Hi"
        "#},
        None::<[_; 0]>,
    )]
    #[case(
        indoc! {r#"
            foo:
              a: "Lorem"
              b: "Ipsum"
              bar:
                a: "Dolor"
                b: "Sit"
                c: "Amet"
            baz:
              a: "Lorem Ipsum"
              b: "Dolor Sit Amet"
        "#},
        Some([
            (
                "foo".into(),
                RawNode::Namespace(RawNamespace {
                    map: HashMap::from_iter([
                        ("a".into(), RawNode::Message("Lorem".into())),
                        ("b".into(), RawNode::Message("Ipsum".into())),
                        (
                            "bar".into(),
                            RawNode::Namespace(RawNamespace {
                                map: HashMap::from_iter([
                                    ("a".into(), RawNode::Message("Dolor".into())),
                                    ("b".into(), RawNode::Message("Sit".into())),
                                    ("c".into(), RawNode::Message("Amet".into())),
                                ]),
                            }),
                        ),
                    ]),
                }),
            ),
            (
                "baz".into(),
                RawNode::Namespace(RawNamespace {
                    map: HashMap::from_iter([
                        ("a".into(), RawNode::Message("Lorem Ipsum".into())),
                        ("b".into(), RawNode::Message("Dolor Sit Amet".into())),
                    ]),
                }),
            ),
        ]),
    )]
    fn read_definition(
        #[case] input: &str,
        #[case] expected_output: Option<impl IntoIterator<Item = (CompactString, RawNode)>>,
    ) {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{input}").unwrap();
        let actual_output = Definition::read(file.path().into()).ok();
        let expected_output = expected_output.map(|pairs| Definition {
            root: RawNamespace {
                map: pairs.into_iter().collect(),
            },
        });
        assert_eq!(actual_output, expected_output);
    }
}

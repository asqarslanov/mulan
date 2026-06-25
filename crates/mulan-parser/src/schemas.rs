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
    #[case(<&str>::default(), Some(iter::empty()))]
    #[case(
        indoc! {r#"
            foo: "Hello"
            bar: "Hi"
        "#},
        Some([
            (
                CompactString::new("foo"),
                RawNode::Message(CompactString::new("Hello")),
            ),
            (
                CompactString::new("bar"),
                RawNode::Message(CompactString::new("Hi")),
            ),
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
                CompactString::new("foo"),
                RawNode::Namespace(RawNamespace {
                    map: [
                        (
                            CompactString::new("a"),
                            RawNode::Message(CompactString::new("Lorem")),
                        ),
                        (
                            CompactString::new("b"),
                            RawNode::Message(CompactString::new("Ipsum")),
                        ),
                        (
                            CompactString::new("bar"),
                            RawNode::Namespace(RawNamespace {
                                map: [
                                    (
                                        CompactString::new("a"),
                                        RawNode::Message(CompactString::new("Dolor")),
                                    ),
                                    (
                                        CompactString::new("b"),
                                        RawNode::Message(CompactString::new("Sit")),
                                    ),
                                    (
                                        CompactString::new("c"),
                                        RawNode::Message(CompactString::new("Amet")),
                                    ),
                                ]
                                .into_iter()
                                .collect(),
                            }),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                }),
            ),
            (
                CompactString::new("baz"),
                RawNode::Namespace(RawNamespace {
                    map: [
                        (
                            CompactString::new("a"),
                            RawNode::Message(CompactString::new("Lorem Ipsum")),
                        ),
                        (
                            CompactString::new("b"),
                            RawNode::Message(CompactString::new("Dolor Sit Amet")),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                }),
            ),
        ]),
    )]
    // more tests on namespaces
    fn read_definition(
        #[case] input: &str,
        #[case] expected_output: Option<impl IntoIterator<Item = (CompactString, RawNode)>>,
    ) {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{input}").unwrap();
        let actual_output = Definition::read(file.path().to_owned()).ok();
        let expected_output = expected_output.map(|pairs| Definition {
            root: RawNamespace {
                map: pairs.into_iter().collect(),
            },
        });
        assert_eq!(actual_output, expected_output);
    }
}

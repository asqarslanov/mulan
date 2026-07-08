//! Defines the [`Input`] struct and the logic to read it from the filesystem.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::{fs, io};

use compact_str::CompactString;
use foldhash::HashMap;
use mitsein::iter1::IntoIterator1;
use mulan_config::Language;
use serde::Deserialize;
use strum::EnumTryAs;

/// A simple collection of locale [`Definition`]s parsed with [`serde`].
///
/// This type is used to quickly map the contents of locale files
/// to Rust values. Later, it will be converted into the more useful
/// [`mulan_parser::Output`](crate::Output) type.
#[derive(Debug)]
pub struct Input {
    /// Maps a language tag to the contents of the corresponding locale.
    ///
    /// May not include all locales specified in [`mulan_config::Config`].
    pub locales: HashMap<Language, Definition>,
}

/// A single-language definition of a locale read from a locale file
/// (e.g., `locales/en-US/locale.yaml`).
///
/// This structure uses basic data types and is constructed
/// with [`serde::Deserialize`]. Later, a more strongly typed tree
/// can be constructed from a collection of [`Definition`]s.
///
/// ## Example Definition
///
/// ```yaml
/// app-name: "Mulan"
/// greeting: "Hello, {name}!"
/// namespace-foo:
///   lorem-upsum: "Dolor sit amet"
/// ```
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Definition {
    /// A locale definition is ultimately a tree of nested namespaces
    /// (see [`RawNamespace`]). The `root` namespace is the outermost
    /// namespace. It is always present, even if the locale definition is empty.
    #[serde(flatten)]
    pub(super) root: RawNamespace,
}

/// A "grouping" of messages to organize them conveniently.
///
/// Subkeys from different namespaces don't collide and can take
/// the same values.
///
/// ```yaml
/// ns1:
///   msg1: "Foo"
///   msg2: "Bar"
/// ns2:
///   msg1: "Lorem"
///   msg2: "Ipsum"
/// ```
///
/// Namespaces can nest to produce more complex hieararchies of messages.
///
/// ```yaml
/// one-namespace:
///   foo: "Lorem"
///   bar: "Ipsum"
///   another-namespace:
///     baz: "Dolor"
/// ```
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RawNamespace {
    /// Maps raw keys to namespace nodes (see [`RawNode`]).
    ///
    /// All nodes within a namespace must have unique keys
    /// (i.e., a message can't have the same key as a sibling namespace).
    #[serde(flatten)]
    pub(super) map: HashMap<CompactString, RawNode>,
}

/// A value in a [namespace](RawNamespace) of a locale [`Definition`].
///
/// Can either be a message template or another namespace.
#[derive(Debug, Deserialize, PartialEq, Eq, EnumTryAs)]
#[serde(untagged)]
pub enum RawNode {
    /// Raw text that will later be properly parsed
    /// to a [`Template`](crate::Template).
    Message(CompactString),

    /// A nested namespace.
    Namespace(RawNamespace),
}

/// Errors of [`Input::read`].
#[derive(Debug)]
pub enum ReadLocaleError {
    /// Failed to read a file.
    Io { path: PathBuf, error: io::Error },

    /// Failed to parse a YAML file according to the schema.
    Format(serde_saphyr::Error),
}

impl Input {
    /// Locates and parses YAML locale definition files to Rust values.
    pub fn read(config: &mulan_config::Config) -> Result<Self, ReadLocaleError> {
        let locales_dir = {
            config
                .source
                .parent()
                .expect("config source should point to a file")
                .join("locales/")
        };
        let locales = {
            config
                .locales
                .iter()
                .map(|&locale| {
                    let path = locales_dir.join(locale.tag()).with_added_extension("yaml");
                    let definition = Definition::read(path.into())?;
                    Ok((locale, definition))
                })
                .collect::<Result<_, _>>()?
        };
        Ok(Self { locales })
    }
}

/// Errors of [`Definition::at`].
#[derive(Debug, PartialEq, Eq)]
pub enum DefinitionAtError {
    /// ...
    NotFound {
        /// ...
        index: usize,
    },

    /// ...
    NotANamespace {
        /// ...
        index: usize,
    },
}

impl Definition {
    /// Parses a YAML locale definition file to a Rust value.
    fn read(path: Cow<'_, Path>) -> Result<Self, ReadLocaleError> {
        let file_contents = fs::read_to_string(&path).map_err(|error| ReadLocaleError::Io {
            error,
            path: path.into_owned(),
        })?;
        serde_saphyr::from_str(&file_contents).map_err(ReadLocaleError::Format)
    }

    /// Returns a reference to the node at the given path.
    ///
    /// For example, let `definition: Definiton` be
    ///
    /// ```yaml
    /// foo:
    ///   a: "Lorem"
    ///   b: "Ipsum"
    ///   bar:
    ///     a: "Dolor"
    ///     b: "Sit"
    ///     c: "Amet"
    /// ```
    ///
    /// Then,
    ///
    /// ```ignore
    /// definition.at(["foo", "a"])
    /// => "Lorem"
    ///
    /// definition.at(["foo", "a", "x"])
    /// => DefinitionAtError::NotANamespace
    ///
    /// definition.at(["foo", "bar"])
    /// => { a: "Dolor", b: "Sit", c: "Amet" }
    ///
    /// definition.at(["foo", "bar", "c"])
    /// => "Amet"
    ///
    /// definition.at(["foo", "doesnt-exist"])
    /// => DefinitionAtError::NotFound
    ///
    /// definition.at(["baz"])
    /// => DefinitionAtError::NotFound
    /// ```
    pub fn at<I>(&self, path: I) -> Result<&RawNode, DefinitionAtError>
    where
        I: IntoIterator1,
        I::Item: AsRef<str>,
        I::IntoIter: DoubleEndedIterator,
    {
        let mut index = 0;
        let mut namespace = &self.root;
        let (subkeys, last_subkey) = path.into_iter1().into_rtail_and_head();
        for subkey in subkeys {
            let node = {
                namespace
                    .map
                    .get(subkey.as_ref())
                    .ok_or(DefinitionAtError::NotFound { index })?
            };
            namespace = {
                node.try_as_namespace_ref()
                    .ok_or(DefinitionAtError::NotANamespace { index })?
            };
            index += 1
        }
        namespace
            .map
            .get(last_subkey.as_ref())
            .ok_or(DefinitionAtError::NotFound { index })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;
    use std::iter;

    use compact_str::CompactString;
    use foldhash::HashMap;
    use indoc::indoc;
    use rstest::rstest;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::chumsky_parse::ChumskyParser as _;
    use crate::identifier::{Identifier, Word};
    use crate::{Key, Subkey};

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

    enum PseudoNode<'a> {
        Message(&'a str),
        Namespace(&'a str),
    }

    #[rstest]
    #[case(
        "foo",
        Ok(PseudoNode::Namespace(indoc! {r#"
            a: "Lorem"
            b: "Ipsum"
            bar:
              a: "Dolor"
              b: "Sit"
              c: "Amet"
        "#})),
    )]
    #[case("foo.a", Ok(PseudoNode::Message("Lorem")))]
    #[case("foo.b", Ok(PseudoNode::Message("Ipsum")))]
    #[case(
        "foo.bar",
        Ok(PseudoNode::Namespace(indoc! {r#"
            a: "Dolor"
            b: "Sit"
            c: "Amet"
        "#})),
    )]
    #[case("foo.bar.a", Ok(PseudoNode::Message("Dolor")))]
    #[case("foo.bar.b", Ok(PseudoNode::Message("Sit")))]
    #[case("foo.bar.c", Ok(PseudoNode::Message("Amet")))]
    #[case(
        "baz",
        Ok(PseudoNode::Namespace(indoc! {r#"
            a: "Lorem Ipsum"
            b: "Dolor Sit Amet"
        "#})),
    )]
    #[case("baz.a", Ok(PseudoNode::Message("Lorem Ipsum")))]
    #[case("baz.b", Ok(PseudoNode::Message("Dolor Sit Amet")))]
    #[case("bar", Err(DefinitionAtError::NotFound { index: 0 }))]
    #[case("foo.a.x", Err(DefinitionAtError::NotANamespace { index: 1 }))]
    #[case("foo.bar.a.x.y", Err(DefinitionAtError::NotANamespace { index: 2 }))]
    #[case("foo.c", Err(DefinitionAtError::NotFound { index: 1 }))]
    #[case("foo.bar.baz", Err(DefinitionAtError::NotFound { index: 2 }))]
    fn definition_at(
        #[case] input: &str,
        #[case] expected_output: Result<PseudoNode<'_>, DefinitionAtError>,
    ) {
        const DEFINITION_RAW: &str = indoc! {r#"
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
        "#};

        let word_parser = Word::chumsky_parser();
        let ident_parser = Identifier::chumsky_parser(&word_parser);
        let subkey_parser = Subkey::chumsky_parser(&ident_parser);
        let key_parser = Key::chumsky_parser(&subkey_parser);
        let key = key_parser.mulan_parse(input).unwrap();
        let definition = {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{DEFINITION_RAW}").unwrap();
            Definition::read(file.path().into()).unwrap()
        };
        let actual_output = definition.at(key.segments.iter1().map(Subkey::to_kebab_case));
        let expected_output = expected_output.map(|node| match node {
            PseudoNode::Message(contents) => RawNode::Message(contents.into()),
            PseudoNode::Namespace(contents) => {
                let mut file = NamedTempFile::new().unwrap();
                write!(file, "{contents}").unwrap();
                let definition = Definition::read(file.path().into()).unwrap();
                RawNode::Namespace(definition.root)
            }
        });
        assert_eq!(
            actual_output,
            match expected_output {
                Ok(ref node) => Ok(node),
                Err(err) => Err(err),
            },
        );
    }
}

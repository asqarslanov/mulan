//! Defines the [`RawLocaleMap`] struct and the logic to read it
//! from the filesystem.

use std::borrow::Cow;
use std::fs;
use std::path::Path;

use compact_str::CompactString;
use foldhash::HashMap;
use mitsein::compact_string1::{CompactString1, CompactString1Ext as _};
use mitsein::vec1::Vec1;
use mulan_config::Language;
use serde::Deserialize;
use strum::EnumTryAs;

use crate::errors::{RawLocaleMapError, ReadFileError, YamlError};

/// A simple collection of locale [definition](RlmDefinition)s
/// parsed with [`mod@serde`].
///
/// This type is used to quickly map the contents of locale files
/// to Rust values. Later, it will be converted into the more useful
/// [`crate::Bundle`] type.
#[derive(Debug)]
pub struct RawLocaleMap {
    /// Maps a language tag to the contents of the corresponding locale.
    ///
    /// May not include all locales specified in [`mulan_config::Config`].
    pub locales: HashMap<Language, RlmDefinition>,
}

/// [`RawLocaleMap`]-definition.
///
/// A single-language definition of a locale read from a locale file
/// (e.g., `locales/en-US/locale.yaml`).
///
/// This structure uses basic data types and is constructed
/// with [`serde::Deserialize`]. Later, a more strongly typed tree
/// can be constructed from a collection of [`RlmDefinition`]s.
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
pub struct RlmDefinition {
    /// A locale definition is ultimately a tree of nested namespaces
    /// (see [`RlmNamespace`]). The `root` namespace is the outermost
    /// namespace. It is always present, even if the locale definition is empty.
    #[serde(flatten)]
    pub(super) root: RlmNamespace,
}

/// [`RawLocaleMap`]-namespace.
///
/// A "grouping" of messages to organize them conveniently.
///
/// Key parts from different namespaces don't collide and can take
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
pub struct RlmNamespace {
    /// Maps raw key parts to namespace nodes (see [`RlmNode`]).
    ///
    /// All nodes within a namespace must have unique keys
    /// (i.e., a message can't have the same key as a sibling namespace).
    #[serde(flatten)]
    pub(super) map: HashMap<CompactString1, RlmNode>,
}

/// [`RawLocaleMap`]-node.
///
/// A value in a [namespace](RlmNamespace)
/// of a locale [definition](RlmDefinition).
///
/// Can either be a message template or another namespace.
#[derive(Debug, Deserialize, PartialEq, Eq, EnumTryAs)]
#[serde(untagged)]
pub enum RlmNode {
    /// Raw text that will later be properly parsed
    /// to a [`Template`](crate::Template).
    Message(CompactString),

    /// A nested namespace.
    Namespace(RlmNamespace),
}

impl RawLocaleMap {
    /// Locates and parses YAML locale definition files to Rust values.
    pub fn from_fs(config: &mulan_config::Config) -> Result<Self, RawLocaleMapError> {
        let locales_dir = config.meta.root_dir.join("locales/");
        let locales = {
            config
                .locales
                .iter()
                .map(|&locale| {
                    let path = {
                        locales_dir
                            .join(locale.tag().as_ref())
                            .with_extension("yaml")
                    };
                    let path = path.to_path(""); // doesn't add a prefix
                    let definition = RlmDefinition::read(path.into())?;
                    Ok((locale, definition))
                })
                .collect::<Result<_, _>>()?
        };
        Ok(Self { locales })
    }
}

/// Represents a path to a node. A "dumber" counterpart to [`crate::DottedKey`].
#[derive(Debug, Clone)]
pub struct RawDottedKey {
    /// For simplicity, key parts are stored as plain strings
    /// rather than wrapped in newtypes with invariants.
    pub(super) parts: Vec1<CompactString1>,
}

impl RawDottedKey {
    /// Returns a dot-separated string representation.
    ///
    /// E.g., `["quick", "brown", "fox"]` will become `"quick.brown.fox"`.
    #[must_use]
    pub fn to_compact_string1(&self) -> CompactString1 {
        (&self.parts).join_compact1(".")
    }
}

/// Errors of [`RlmDefinition::at`].
#[derive(Debug, PartialEq, Eq)]
pub enum RlmDefinitionAtError {
    /// The path doesn't exist.
    NotFound {
        /// The index (0-based) of the first key part we couldn't find.
        index: usize,
    },

    /// Tried to access a key part as a namespace, but it turned out
    /// to point at a message.
    NotANamespace {
        /// The index (0-based) of the misinterpreted key part.
        index: usize,
    },
}

impl RlmDefinition {
    /// Parses a YAML locale definition file to a Rust value.
    fn read(path: Cow<'_, Path>) -> Result<Self, RawLocaleMapError> {
        let file_contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) => {
                let path = path.into_owned();
                return Err(RawLocaleMapError::ReadFile(ReadFileError { path, error }));
            }
        };
        serde_saphyr::from_str(&file_contents).map_err(|err| {
            RawLocaleMapError::Yaml(YamlError {
                inner: Box::new(err),
                filename: path.into_owned(),
                source_code: file_contents,
            })
        })
    }

    /// Returns a reference to the node at the given path.
    ///
    /// For example, let `definition: RlmDefiniton` be
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
    /// => RlmDefinitionAtError::NotANamespace
    ///
    /// definition.at(["foo", "bar"])
    /// => { a: "Dolor", b: "Sit", c: "Amet" }
    ///
    /// definition.at(["foo", "bar", "c"])
    /// => "Amet"
    ///
    /// definition.at(["foo", "doesnt-exist"])
    /// => RlmDefinitionAtError::NotFound
    ///
    /// definition.at(["baz"])
    /// => RlmDefinitionAtError::NotFound
    /// ```
    pub fn at(&self, path: &RawDottedKey) -> Result<&RlmNode, RlmDefinitionAtError> {
        let mut index = 0;
        let mut namespace = &self.root;
        let (key_parts, last_key_part) = path.parts.iter1().into_rtail_and_head();
        for key_part in key_parts {
            let node = {
                namespace
                    .map
                    .get(key_part.as_str())
                    .ok_or(RlmDefinitionAtError::NotFound { index })?
            };
            namespace = {
                node.try_as_namespace_ref()
                    .ok_or(RlmDefinitionAtError::NotANamespace { index })?
            };
            index += 1;
        }
        namespace
            .map
            .get(last_key_part.as_str())
            .ok_or(RlmDefinitionAtError::NotFound { index })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;
    use std::iter;

    use foldhash::HashMap;
    use indoc::indoc;
    use mitsein::str1;
    use mulan_config::Case;
    use rstest::rstest;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::DottedKey;
    use crate::chumsky_parse::ChumskyParser as _;
    use crate::identifier::{Identifier, Word};

    #[rstest]
    #[case(<&str>::default(), Some(iter::empty()))]
    #[case(
        indoc! {r#"
            foo: "Hello"
            bar: "Hi"
        "#},
        Some([
            (str1!("foo").into(), RlmNode::Message("Hello".into())),
            (str1!("bar").into(), RlmNode::Message("Hi".into())),
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
                str1!("foo").into(),
                RlmNode::Namespace(RlmNamespace {
                    map: HashMap::from_iter([
                        (str1!("a").into(), RlmNode::Message("Lorem".into())),
                        (str1!("b").into(), RlmNode::Message("Ipsum".into())),
                        (
                            str1!("bar").into(),
                            RlmNode::Namespace(RlmNamespace {
                                map: HashMap::from_iter([
                                    (str1!("a").into(), RlmNode::Message("Dolor".into())),
                                    (str1!("b").into(), RlmNode::Message("Sit".into())),
                                    (str1!("c").into(), RlmNode::Message("Amet".into())),
                                ]),
                            }),
                        ),
                    ]),
                }),
            ),
            (
                str1!("baz").into(),
                RlmNode::Namespace(RlmNamespace {
                    map: HashMap::from_iter([
                        (str1!("a").into(), RlmNode::Message("Lorem Ipsum".into())),
                        (str1!("b").into(), RlmNode::Message("Dolor Sit Amet".into())),
                    ]),
                }),
            ),
        ]),
    )]
    fn read_definition(
        #[case] input: &str,
        #[case] expected_output: Option<impl IntoIterator<Item = (CompactString1, RlmNode)>>,
    ) {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{input}").unwrap();
        let actual_output = RlmDefinition::read(file.path().into()).ok();
        let expected_output = expected_output.map(|pairs| RlmDefinition {
            root: RlmNamespace {
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
    #[case("bar", Err(RlmDefinitionAtError::NotFound { index: 0 }))]
    #[case("foo.a.x", Err(RlmDefinitionAtError::NotANamespace { index: 1 }))]
    #[case("foo.bar.a.x.y", Err(RlmDefinitionAtError::NotANamespace { index: 2 }))]
    #[case("foo.c", Err(RlmDefinitionAtError::NotFound { index: 1 }))]
    #[case("foo.bar.baz", Err(RlmDefinitionAtError::NotFound { index: 2 }))]
    fn definition_at(
        #[case] input: &str,
        #[case] expected_output: Result<PseudoNode<'_>, RlmDefinitionAtError>,
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
        let key_parser = DottedKey::chumsky_parser(&ident_parser);
        let key = key_parser.mulan_parse(input).unwrap();
        let definition = {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{DEFINITION_RAW}").unwrap();
            RlmDefinition::read(file.path().into()).unwrap()
        };
        let key = RawDottedKey {
            parts: {
                key.parts
                    .iter1()
                    .map(|part| part.to_compact_string1(Case::Kebab))
                    .collect1()
            },
        };
        let actual_output = definition.at(&key);
        let expected_output = expected_output.map(|node| match node {
            PseudoNode::Message(contents) => RlmNode::Message(contents.into()),
            PseudoNode::Namespace(contents) => {
                let mut file = NamedTempFile::new().unwrap();
                write!(file, "{contents}").unwrap();
                let definition = RlmDefinition::read(file.path().into()).unwrap();
                RlmNode::Namespace(definition.root)
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

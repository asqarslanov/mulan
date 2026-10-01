//! Defines the [`LocaleMap`] struct and the logic to read it
//! from the filesystem.

use std::borrow::Cow;
use std::fs;
use std::path::Path;

use compact_str::CompactString;
use foldhash::{HashMap, HashMapExt as _};
use mitsein::compact_string1::CompactString1;
use mitsein::vec1::Vec1;
use mulan_config::Language;
use serde::Deserialize;
use strum::EnumTryAs;

use crate::chumsky_parse::ChumskyParser;
use crate::errors::{
    InvalidKeyError, InvalidSyntaxError, InvalidTemplateError, LocaleMapError, ReadFileError,
    YamlError,
};
use crate::identifier::Word;
use crate::{DottedKey, Identifier, Tag, Template, TemplatePart};

/// A simple collection of locale [`LDefinition`]s parsed with [`serde`].
///
/// This type is used to quickly map the contents of locale files
/// to Rust values. Later, it will be converted into the more useful
/// [`crate::Bundle`] type.
#[derive(Debug)]
pub struct LocaleMap {
    /// Maps a language tag to the contents of the corresponding locale.
    ///
    /// May not include all locales specified in [`mulan_config::Config`].
    pub locales: HashMap<Language, LDefinition>,
}

/// [`LDefinition`]'s deserializer.
#[derive(Debug, PartialEq, Eq, Deserialize)]
struct RawDefinition {
    /// Maps to [`LDefinition::root`].
    #[serde(flatten)]
    root: RawNamespace,
}

/// A strongly-typed single-language definition of a locale
/// (read from a locale file such as `locales/en-US/locale.yaml`).
///
/// Loosely-typed counterpart: [`RawDefinition`].
///
/// ## Example Definition
///
/// ```yaml
/// app-name: "Mulan"
/// greeting: "Hello, {name}!"
/// namespace-foo:
///   lorem-upsum: "Dolor sit amet"
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct LDefinition {
    /// A locale definition is ultimately a tree of nested namespaces.
    /// The `root` namespace is the outermost namespace.
    /// It is always present, even if the locale definition is empty.
    pub(super) root: LNamespace,
}

impl LDefinition {
    ///
    fn from_fs(locales_dir: &Path, locale: Language) -> Result<Self, LocaleMapError> {
        let path = {
            locales_dir
                .join(locale.tag().as_ref())
                .with_extension("yaml")
        };
        let raw_definition = RawDefinition::read(path.into())?;
        Self::from_raw(&raw_definition, locale).map_err(LocaleMapError::InvalidSyntax)
    }

    ///
    fn from_raw(
        raw_definition: &RawDefinition,
        locale: Language,
    ) -> Result<Self, InvalidSyntaxError> {
        let word_parser = Word::chumsky_parser();
        let ident_parser = Identifier::chumsky_parser(&word_parser);
        let _key_parser = DottedKey::chumsky_parser(&ident_parser);
        let tag_parser = Tag::chumsky_parser(&ident_parser);
        let template_part_parser = TemplatePart::chumsky_parser(&tag_parser);
        let template_parser = Template::chumsky_parser(&template_part_parser);
        let root = LNamespace::from_raw(
            &raw_definition.root,
            locale,
            None,
            &ident_parser,
            &template_parser,
        )?;
        Ok(Self { root })
    }

    /// Returns a reference to the node at the given path.
    ///
    /// For example, let `definition: LDefiniton` be
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
    pub fn at(&self, path: &DottedKey) -> Result<&LNode, LDefinitionAtError> {
        let mut index = 0;
        let mut namespace = &self.root;
        let (key_parts, last_key_part) = path.parts.iter1().into_rtail_and_head();
        for key_part in key_parts {
            let node = {
                namespace
                    .map
                    .get(key_part)
                    .ok_or(LDefinitionAtError::NotFound { index })?
            };
            namespace = {
                node.try_as_namespace_ref()
                    .ok_or(LDefinitionAtError::NotANamespace { index })?
            };
            index += 1;
        }
        namespace
            .map
            .get(last_key_part)
            .ok_or(LDefinitionAtError::NotFound { index })
    }
}

/// [`LNamespace`]'s deserializer.
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub(super) struct RawNamespace {
    /// Maps to [`LNamespace::map`].
    #[serde(flatten)]
    map: HashMap<CompactString1, RawNode>,
}

/// A strongly-typed "grouping" of messages to organize them conveniently.
///
/// Key parts from different namespaces don't collide and can take
/// the same values.
///
/// Loosely-typed counterpart: [`RawNamespace`].
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
#[derive(Debug, PartialEq, Eq)]
pub struct LNamespace {
    /// Maps raw key parts to namespace nodes (see [`LNode`]).
    ///
    /// All nodes within a namespace must have unique keys
    /// (i.e., a message can't have the same key as a sibling namespace).
    pub(super) map: HashMap<Identifier, LNode>,
}

impl LNamespace {
    fn from_raw<'input>(
        raw: &'input RawNamespace,
        locale: Language,
        parent_key: Option<&DottedKey>,
        ident_parser: &impl ChumskyParser<'input, Identifier>,
        template_parser: &impl ChumskyParser<'input, Template>,
    ) -> Result<Self, InvalidSyntaxError> {
        let mut map = HashMap::new();
        for (key_raw, node_raw) in &raw.map {
            let key_part = ident_parser.mulan_parse(key_raw).map_err(|errors| {
                InvalidSyntaxError::InvalidKey(InvalidKeyError {
                    locale,
                    parent_key: parent_key.cloned(),
                    errors,
                })
            })?;
            let rtail = parent_key.map_or_default(|key| key.parts.to_vec());
            let key = DottedKey {
                parts: Vec1::from_rtail_and_head(rtail, key_part.clone()),
            };
            let l_node = LNode::from_raw(node_raw, locale, &key, ident_parser, template_parser)?;
            map.insert(key_part, l_node);
        }
        Ok(Self { map })
    }
}

/// [`LNode`]'s deserializer.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum RawNode {
    /// Maps to [`LNode::Message`].
    Message(CompactString),

    /// Maps to [`LNode::Namespace`].
    Namespace(RawNamespace),
}

/// A value in an [`LNamespace`] in an [`LDefinition`].
///
/// Can either be a message template or another namespace.
///
/// Loosely-typed counterpart: [`RawNode`].
#[derive(Debug, EnumTryAs, PartialEq, Eq)]
pub enum LNode {
    /// Raw text that will later be properly parsed to a [`Template`].
    Message(Template),

    /// A nested namespace.
    Namespace(LNamespace),
}

impl LNode {
    fn from_raw<'input>(
        raw: &'input RawNode,
        locale: Language,
        key: &DottedKey,
        ident_parser: &impl ChumskyParser<'input, Identifier>,
        template_parser: &impl ChumskyParser<'input, Template>,
    ) -> Result<Self, InvalidSyntaxError> {
        let l_node = match raw {
            RawNode::Message(msg_raw) => {
                LNode::Message(template_parser.mulan_parse(msg_raw).map_err(|errors| {
                    InvalidSyntaxError::InvalidTemplate(InvalidTemplateError {
                        locale,
                        key: key.clone(),
                        errors,
                    })
                })?)
            }
            RawNode::Namespace(ns_raw) => LNode::Namespace(LNamespace::from_raw(
                ns_raw,
                locale,
                Some(key),
                ident_parser,
                template_parser,
            )?),
        };
        Ok(l_node)
    }
}

impl LocaleMap {
    /// Locates and parses YAML locale definition files to Rust values.
    pub fn from_fs(config: &mulan_config::Config) -> Result<Self, LocaleMapError> {
        let locales_dir = config.meta.root_dir.join("locales/");
        let locales = {
            config
                .locales
                .iter()
                .map(|&locale| {
                    let l_definition = LDefinition::from_fs(&locales_dir.to_path(""), locale)?;
                    Ok((locale, l_definition))
                })
                .collect::<Result<_, _>>()?
        };
        Ok(Self { locales })
    }
}

/// Errors of [`LDefinition::at`].
#[derive(Debug, PartialEq, Eq)]
pub enum LDefinitionAtError {
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

impl RawDefinition {
    /// Parses a YAML locale definition file to a Rust value.
    fn read(path: Cow<'_, Path>) -> Result<Self, LocaleMapError> {
        let file_contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) => {
                let path = path.into_owned();
                return Err(LocaleMapError::ReadFile(ReadFileError { path, error }));
            }
        };
        serde_saphyr::from_str(&file_contents).map_err(|err| {
            LocaleMapError::Yaml(YamlError {
                inner: Box::new(err),
                filename: path.into_owned(),
                source_code: file_contents,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;
    use std::iter;

    use indoc::indoc;
    use mitsein::str1;
    use rstest::rstest;
    use tempfile::NamedTempFile;

    use super::*;

    #[rstest]
    #[case(<&str>::default(), Some(iter::empty()))]
    #[case(
        indoc! {r#"
            foo: "Hello"
            bar: "Hi"
        "#},
        Some([
            (str1!("foo").into(), RawNode::Message("Hello".into())),
            (str1!("bar").into(), RawNode::Message("Hi".into())),
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
                RawNode::Namespace(RawNamespace {
                    map: HashMap::from_iter([
                        (str1!("a").into(), RawNode::Message("Lorem".into())),
                        (str1!("b").into(), RawNode::Message("Ipsum".into())),
                        (
                            str1!("bar").into(),
                            RawNode::Namespace(RawNamespace {
                                map: HashMap::from_iter([
                                    (str1!("a").into(), RawNode::Message("Dolor".into())),
                                    (str1!("b").into(), RawNode::Message("Sit".into())),
                                    (str1!("c").into(), RawNode::Message("Amet".into())),
                                ]),
                            }),
                        ),
                    ]),
                }),
            ),
            (
                str1!("baz").into(),
                RawNode::Namespace(RawNamespace {
                    map: HashMap::from_iter([
                        (str1!("a").into(), RawNode::Message("Lorem Ipsum".into())),
                        (str1!("b").into(), RawNode::Message("Dolor Sit Amet".into())),
                    ]),
                }),
            ),
        ]),
    )]
    fn read_definition(
        #[case] input: &str,
        #[case] expected_output: Option<impl IntoIterator<Item = (CompactString1, RawNode)>>,
    ) {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{input}").unwrap();
        let actual_output = RawDefinition::read(file.path().into()).ok();
        let expected_output = expected_output.map(|pairs| RawDefinition {
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
    #[case("bar", Err(LDefinitionAtError::NotFound { index: 0 }))]
    #[case("foo.a.x", Err(LDefinitionAtError::NotANamespace { index: 1 }))]
    #[case("foo.bar.a.x.y", Err(LDefinitionAtError::NotANamespace { index: 2 }))]
    #[case("foo.c", Err(LDefinitionAtError::NotFound { index: 1 }))]
    #[case("foo.bar.baz", Err(LDefinitionAtError::NotFound { index: 2 }))]
    fn definition_at(
        #[case] input: &str,
        #[case] expected_output: Result<PseudoNode<'_>, LDefinitionAtError>,
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
        let tag_parser = Tag::chumsky_parser(&ident_parser);
        let template_part_parser = TemplatePart::chumsky_parser(&tag_parser);
        let template_parser = Template::chumsky_parser(&template_part_parser);
        let definition = {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{DEFINITION_RAW}").unwrap();
            let raw_definition = RawDefinition::read(file.path().into()).unwrap();
            LDefinition::from_raw(&raw_definition, Language::EnUs).unwrap()
        };
        let actual_output = definition.at(&key);
        let expected_output = expected_output.map(|node| match node {
            PseudoNode::Message(contents) => {
                LNode::Message(template_parser.mulan_parse(contents).unwrap())
            }
            PseudoNode::Namespace(contents) => {
                let mut file = NamedTempFile::new().unwrap();
                write!(file, "{contents}").unwrap();
                let raw_definition = RawDefinition::read(file.path().into()).unwrap();
                let l_definition = LDefinition::from_raw(&raw_definition, Language::EnUs).unwrap();
                LNode::Namespace(l_definition.root)
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

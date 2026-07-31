//! Defines the [`Output`] struct and its methods.

use std::collections::BTreeMap;

use mitsein::compact_string1::CompactString1;
use mitsein::vec1::Vec1;
use mulan_config::Language;

use crate::identifier::Identifier;
use crate::template::Template;

/// All messages from all user locales, strictly-typed, validated, and
/// organized. The final parsing result used to generate locale bindings.
/// Its structure is based on the main locale.
///
/// Use [`.iter()`](Self::iter) to traverse through [`Node`]s.
/// Message nodes store all translations alongside each other.
/// Data is stored alphabetically to ensure deterministic output.
#[derive(Debug)]
pub struct Output {
    /// [`Output`] is ultimately a tree of nested namespaces
    /// (see [`Namespace`]). The `root` namespace is the outermost namespace.
    /// It is always present, even if the main locale definition is empty.
    pub(super) root: Namespace,
}

/// A "grouping" of messages to organize them conveniently.
///
/// [`Subkey`]s from different namespaces don't collide and can take
/// the same values.
///
/// See [`RawNamespace`](crate::schemas::input::RawNamespace)
/// for visual examples.
#[derive(Debug)]
pub struct Namespace {
    /// Maps raw keys to namespace nodes (see [`Node`]).
    ///
    /// All nodes within a namespace must have unique keys
    /// (i.e., a message can't have the same key as a sibling namespace).
    pub(super) map: BTreeMap<Subkey, Node>,
}

/// A single segment of a message [`Key`].
///
/// E.g., the key `frontend.user-settings.account` has the [`Subkey`]s
/// `frontend`, `user-settings`, `account`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subkey {
    value: Identifier,
}

impl Subkey {
    /// Converts this subkey to a kebab-case string (e.g., `user1-settings`).
    #[must_use]
    pub fn to_kebab_case(&self) -> CompactString1 {
        self.value.to_kebab_case()
    }
}

/// A full path to a [`Node`] composed of one or more [`Subkey`]s.
///
/// E.g., the [`Key`] `frontend.user-settings.account` has the subkeys
/// `frontend`, `user-settings`, `account`.
#[derive(Debug, PartialEq, Eq)]
pub struct Key {
    pub(crate) segments: Vec1<Subkey>,
}

/// A value in a [`Namespace`].
///
/// Can either be a message template's [`Translations`] or another namespace.
#[derive(Debug)]
pub enum Node {
    /// All translations of a single message.
    Message(Translations),

    /// A nested namespace.
    Namespace(Namespace),
}

/// All user-defined translations of a single message.
///
/// Data is stored alphabetically to ensure deterministic output.
/// The main translation is always present.
#[derive(Debug)]
pub struct Translations {
    /// The message written in the main locale.
    pub(super) main: Template,

    /// Other translations of the message.
    ///
    /// May not include all locales specified in [`mulan_config::Config`].
    pub(super) others: BTreeMap<Language, Template>,
}

/// Defines parsers with [`mod@chumsky`].
mod parser {
    use chumsky::prelude::*;

    use super::{Key, Subkey};
    use crate::chumsky_parse::ChumskyParser;
    use crate::identifier::Identifier;

    impl Key {
        #[must_use]
        pub(crate) fn chumsky_parser<'src>(
            subkey_parser: &impl ChumskyParser<'src, Subkey>,
        ) -> impl ChumskyParser<'src, Self> {
            subkey_parser
                .separated_by(just('.'))
                .at_least(1)
                .collect()
                .map(|segments: Vec<_>| Self {
                    segments: segments.try_into().expect(".at_least(1)"),
                })
        }
    }

    impl Subkey {
        #[must_use]
        pub(crate) fn chumsky_parser<'src>(
            ident_parser: &impl ChumskyParser<'src, Identifier>,
        ) -> impl ChumskyParser<'src, Self> {
            ident_parser.map(|value| Self { value })
        }
    }
}

#[cfg(test)]
mod tests {
    use mitsein::iter1::IteratorExt as _;
    use rstest::rstest;

    use super::*;
    use crate::chumsky_parse::ChumskyParser as _;
    use crate::identifier::Word;

    #[rstest]
    #[case("", None)]
    #[case(" ", None)]
    #[case(".", None)]
    #[case(".a", None)]
    #[case("a.", None)]
    #[case("a ", None)]
    #[case(" a", None)]
    #[case("a", Some(["a"].as_slice()))]
    #[case("a1", Some(["a1"].as_slice()))]
    #[case("foo.bar", Some(["foo", "bar"].as_slice()))]
    #[case("foo1.bar2", Some(["foo1", "bar2"].as_slice()))]
    #[case("foo1.bar2.baz", Some(["foo1", "bar2", "baz"].as_slice()))]
    #[case("foo-1.bar-2", None)]
    #[case("foo1. bar2", None)]
    #[case("foo1 .bar2", None)]
    #[case("foo1 bar2", None)]
    fn parse_key(#[case] input: &str, #[case] expected_output: Option<&[&str]>) {
        let word_parser = Word::chumsky_parser();
        let ident_parser = Identifier::chumsky_parser(&word_parser);
        let subkey_parser = Subkey::chumsky_parser(&ident_parser);
        let key_parser = Key::chumsky_parser(&subkey_parser);
        let actual_output = key_parser.mulan_parse(input).ok();
        let expected_output = expected_output.map(|raw_segments| {
            let segments = {
                raw_segments
                    .iter()
                    .try_into_iter1()
                    .unwrap()
                    .map(|subkey| subkey_parser.mulan_parse(subkey).unwrap())
                    .collect1()
            };
            Key { segments }
        });
        assert_eq!(actual_output, expected_output);
    }
}

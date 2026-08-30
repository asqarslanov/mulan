//! Defines the [`Bundle`] struct and its methods.

use std::collections::BTreeMap;

use indoc::formatdoc;
use mitsein::btree_set1::BTreeSet1;
use mitsein::iter1::{Iterator1, IteratorExt as _};
use mitsein::string1::String1;
use mitsein::vec1::Vec1;
use mulan_config::Language;

use crate::{DottedKey, Identifier, Template};

/// All messages from all user locales, strictly-typed, validated, and
/// organized. The final parsing result used to generate locale bindings.
/// Its structure is based on the main locale.
///
/// Use [`.root.iter()`](Namespace::iter) to traverse through [`Node`]s.
/// Message nodes store all translations alongside each other.
/// Data is stored alphabetically to ensure deterministic output.
#[derive(Debug)]
pub struct Bundle {
    /// [`Bundle`] is ultimately a tree of nested namespaces
    /// (see [`Namespace`]). The `root` namespace is the outermost namespace.
    /// It is always present, even if the main locale definition is empty.
    pub root: Namespace,
}

/// A "grouping" of messages to organize them conveniently.
///
/// Key parts from different namespaces don't collide and can take
/// the same values.
///
/// See [`RawNamespace`](crate::schemas::locale_map::RawNamespace)
/// for visual examples.
#[derive(Debug)]
pub struct Namespace {
    /// Maps key parts to namespace nodes (see [`Node`]).
    ///
    /// All nodes within a namespace must have unique keys
    /// (i.e., a message can't have the same key as a sibling namespace).
    pub(super) map: BTreeMap<Identifier, Node>,
}

impl Namespace {
    /// Returns an iterator over all nodes of this namespace with their
    /// corresponding [`DottedKey`].
    ///
    /// To return these keys, you are required to pass the key of the parent
    /// namespace. Pass `None` if you are iterating over the root namespace.
    pub fn iter(
        &self,
        parent_path: Option<&DottedKey>,
    ) -> impl Iterator<Item = (DottedKey, &Node)> {
        let rtail = parent_path.map(|k| k.parts.to_vec()).unwrap_or_default();
        self.map.iter().map(move |(key_part, node)| {
            let parts = Vec1::from_rtail_and_head(rtail.clone(), key_part.clone());
            let key = DottedKey { parts };
            (key, node)
        })
    }
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
    pub main: Template,

    /// Other translations of the message.
    ///
    /// May not include all locales specified in [`mulan_config::Config`].
    pub others: BTreeMap<Language, Template>,
}

impl Translations {
    /// Returns a preview of the main translation in Markdown.
    ///
    /// ````txt
    /// ```mulan
    /// Hello, {name}!
    /// ```
    /// ````
    #[must_use]
    pub fn markdown_preview(&self, config: &mulan_config::Config) -> Option<String1> {
        let preview = self.main.preview(config)?;
        let backticks_n = self.main.max_consecutive_backticks().max(2) + 1;
        Some(
            formatdoc! {"
                {backticks}mulan
                {preview}
                {backticks}\
                ",
                backticks = "`".repeat(backticks_n),
            }
            .try_into()
            .expect("non-empty"),
        )
    }

    /// The set of all parameters this message requires.
    #[must_use]
    pub fn parameter_set(&self) -> Option<BTreeSet1<&Identifier>> {
        self.main
            .parameter_iter()
            .try_into_iter1()
            .ok()
            .map(Iterator1::collect1)
    }
}

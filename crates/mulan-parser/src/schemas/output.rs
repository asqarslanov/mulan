//! See [`Output`].

use std::collections::BTreeMap;

use mulan_config::Language;

use crate::identifier::Identifier;
use crate::template::Template;

/// All messages from all user locales, strictly-typed, validated, and
/// organized. The final parsing result used to generate locale bindings.
///
/// Use [`.iter()`](Self::iter) to traverse through [`Node`]s.
/// Message nodes store all translations alongside each other.
#[derive(Debug)]
pub struct Output {
    /// [`Output`] is ultimately a tree of nested namespaces
    /// (see [`Namespace`]). The `root` namespace is the outermost namespace.
    /// It is always present, even if the locale definition is empty.
    root: Namespace,
}

/// A "grouping" of messages to organize them conveniently.
///
/// Keys from different namespaces don't collide and can take the same values.
///
/// See [`RawNamespace`](crate::schemas::input::RawNamespace)
/// for visual examples.
#[derive(Debug)]
pub struct Namespace {
    /// Maps raw keys to namespace nodes (see [`Node`]).
    ///
    /// All nodes within a namespace must have unique keys
    /// (i.e., a message can't have the same key as a sibling namespace).
    map: BTreeMap<Key, Node>,
}

/// ...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    /// ...
    value: Identifier,
}

/// ...
#[derive(Debug)]
pub enum Node {
    /// ...
    Message(Translations),

    /// ...
    Namespace(Namespace),
}

/// ...
#[derive(Debug)]
pub struct Translations {
    /// ...
    default: Template,

    /// ...
    others: BTreeMap<Language, Template>,
}

//! See [`Output`].

use std::collections::BTreeMap;

use mulan_config::Language;

use crate::identifier::Identifier;
use crate::template::Template;

/// The final parsing result used to generate locale bindings.
/// Strictly-typed and validated.
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

/// ...
#[derive(Debug)]
pub struct Namespace {
    /// ...
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

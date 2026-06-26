//! See [`Output`].

use std::collections::BTreeMap;

use mulan_config::Language;

use crate::identifier::Identifier;
use crate::template::Template;

/// ...
#[derive(Debug)]
pub struct Output {
    /// ...
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

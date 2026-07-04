//! Defines the [`Output`] struct and its methods.

use std::collections::BTreeMap;

use mitsein::small_vec1::SmallVec1;
use mulan_config::Language;

use crate::identifier::Identifier;
use crate::template::Template;

/// All messages from all user locales, strictly-typed, validated, and
/// organized. The final parsing result used to generate locale bindings.
/// Its structure is based on the default locale.
///
/// Use [`.iter()`](Self::iter) to traverse through [`Node`]s.
/// Message nodes store all translations alongside each other.
/// Data is stored alphabetically to ensure deterministic output.
#[derive(Debug)]
pub struct Output {
    /// [`Output`] is ultimately a tree of nested namespaces
    /// (see [`Namespace`]). The `root` namespace is the outermost namespace.
    /// It is always present, even if the default locale definition is empty.
    pub(super) root: Namespace,
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

/// A path segment of a message.
///
/// E.g., the path `frontend.user-settings.account` has [`Key`]s
/// `frontend`, `user-settings`, `account`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    value: Identifier,
}

// ...
#[derive(Debug)]
pub struct CompoundKey {
    path: SmallVec1<[Key; 2]>,
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
/// The default translation is always present.
#[derive(Debug)]
pub struct Translations {
    /// The message written in the default locale.
    default: Template,

    /// Other translations of the message.
    ///
    /// May not include all locales specified in [`mulan_config::Config`].
    others: BTreeMap<Language, Template>,
}

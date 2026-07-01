//! See [`Input`].

use compact_str::CompactString;
use foldhash::HashMap;
use mulan_config::Language;
use serde::Deserialize;

/// A simple collection of locale [`Definition`]s parsed with [`serde`].
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
/// Keys from different namespaces don't collide and can take the same values.
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

/// A value in a locale [`Definition`]. Can either be a message template
/// or a namespace.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RawNode {
    /// Raw text that will later be properly parsed
    /// to a [`Template`](crate::Template).
    Message(CompactString),

    /// A nested namespace.
    Namespace(RawNamespace),
}

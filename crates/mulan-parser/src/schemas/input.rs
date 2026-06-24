//! See [`Input`].

use compact_str::CompactString;
use foldhash::HashMap;
use mulan_config::Language;
use serde::Deserialize;

/// ...
#[derive(Debug)]
pub struct Input {
    /// ...
    pub default_locale: Definition,

    /// ...
    pub other_locales: HashMap<Language, Definition>,
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
#[derive(Debug, Deserialize)]
pub struct Definition {
    /// A locale definition is ultimately a tree of nested namespaces
    /// (see [`RawNamespace`]). The **root namespace** is the outermost
    /// namespace. It is always present, even if the locale definition is empty.
    #[serde(flatten)]
    root: RawNamespace,
}

/// ...
#[derive(Debug, Deserialize)]
pub struct RawNamespace {
    /// ...
    #[serde(flatten)]
    map: HashMap<CompactString, RawNode>,
}

/// ...
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawNode {
    /// ...
    Message(CompactString),

    /// ...
    Namespace(RawNamespace),
}

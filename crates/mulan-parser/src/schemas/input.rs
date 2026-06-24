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

/// ...
#[derive(Debug, Deserialize)]
pub struct Definition {
    /// ...
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

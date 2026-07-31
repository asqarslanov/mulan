//! Error types.

use std::io;
use std::path::PathBuf;
use std::range::Range;

use compact_str::CompactString;
use mitsein::btree_set1::BTreeSet1;
use mitsein::small_vec1::SmallVec1;
use mitsein::vec1::Vec1;
use mulan_config::Language;

use crate::Parameter;

/// Errors of [`crate::compose`].
#[derive(Debug)]
pub enum ComposeError {
    /// Failed to build a [`crate::schemas::input::Input`].
    Read(ReadInputError),

    /// Failed to build a [`crate::Output`].
    Transform(TransformError),
}

/// Errors of [`crate::schemas::input::Input::read`].
#[derive(Debug)]
pub enum ReadInputError {
    /// Failed to read a file.
    Io { path: PathBuf, error: io::Error },

    /// Failed to parse a YAML file according to the schema.
    Format(serde_saphyr::Error),
}

/// Errors of [`crate::schemas::transform`].
#[derive(Debug)]
pub enum TransformError {
    /// ...
    LocaleNotFound(Language),

    /// ...
    InvalidSubkey {
        locale: Language,

        /// ...
        path: Vec<CompactString>,

        errors: ChumskyAllErrors,
    },

    /// ...
    InvalidTemplate {
        locale: Language,
        key: Vec1<CompactString>,
        errors: ChumskyAllErrors,
    },

    /// ...
    NotANamespace {
        locale: Language,

        /// ...
        key: Vec1<CompactString>,

        /// ...
        index: usize,
    },

    /// ...
    NotAMessage {
        locale: Language,
        key: Vec1<CompactString>,
    },

    /// ...
    UnknownParameters {
        locale: Language,
        key: Vec1<CompactString>,
        parameters: BTreeSet1<Parameter>,
    },
}

/// ...
#[derive(Debug)]
pub struct ChumskyAllErrors {
    /// ...
    pub source: CompactString,

    /// ...
    pub errors: SmallVec1<[ChumskySingleError; 1]>,
}

/// ...
#[derive(Debug)]
pub struct ChumskySingleError {
    /// ...
    pub message: CompactString,

    /// ...
    pub span: Range<usize>,
}

//! Error types.

use std::io;
use std::path::PathBuf;
use std::range::Range;

use compact_str::CompactString;
use mitsein::btree_set1::BTreeSet1;
use mitsein::small_vec1::SmallVec1;
use mitsein::vec1::Vec1;
use mulan_config::Language;
use smallvec::SmallVec;

use crate::Parameter;

/// ...
#[derive(Debug)]
pub enum ComposeError {
    Read(ReadLocaleError),
    Transform(TransformError),
}

/// ...
#[derive(Debug)]
pub enum ReadLocaleError {
    /// Failed to read a file.
    Io { path: PathBuf, error: io::Error },

    /// Failed to parse a YAML file according to the schema.
    Format(serde_saphyr::Error),
}

/// ...
#[derive(Debug)]
pub enum TransformError {
    /// ...
    LocaleNotFound(Language),

    /// ...
    InvalidSubkey {
        locale: Language,

        /// ...
        path: SmallVec<[CompactString; 1]>,

        errors: ChumskyAllErrors,
    },

    /// ...
    InvalidTemplate {
        locale: Language,
        key: SmallVec1<[CompactString; 1]>,
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
        key: SmallVec1<[CompactString; 1]>,
    },

    /// ...
    UnknownParameters {
        locale: Language,
        key: SmallVec1<[CompactString; 1]>,
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

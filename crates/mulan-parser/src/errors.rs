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
    /// The [`Input`] does not have a locale specified in the config.
    LocaleNotFound(Language),

    /// A [`crate::Subkey`] was not parsed successfully (wrong syntax).
    InvalidSubkey {
        locale: Language,

        /// ...
        parent_key: Option<Vec1<CompactString>>,

        errors: ChumskyAllErrors,
    },

    /// A [`crate::Template`] was not parsed succesfully (wrong syntax).
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

    /// A translation of a message has parameters not specified
    /// in the main translation of this message.
    UnknownParameters {
        locale: Language,
        key: Vec1<CompactString>,
        parameters: BTreeSet1<Parameter>,
    },
}

/// The error type of [`crate::chumsky_parse::ChumskyParser::mulan_parse`].
#[derive(Debug)]
pub struct ChumskyAllErrors {
    /// The original string we were trying to parse.
    pub source: CompactString,

    pub errors: SmallVec1<[ChumskySingleError; 1]>,
}

/// A single error in [`ChumskyAllErrors`].
#[derive(Debug)]
pub struct ChumskySingleError {
    /// A description of the error.
    pub message: CompactString,

    /// The problematic byte indices in the `source` string.
    pub span: Range<usize>,
}

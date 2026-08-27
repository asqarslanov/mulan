//! Error types.

use std::io;
use std::path::PathBuf;
use std::range::Range;

use compact_str::CompactString;
use mitsein::btree_set1::BTreeSet1;
use mitsein::small_vec1::SmallVec1;
use mulan_config::Language;

use crate::{Identifier, RawDottedKey};

/// Errors of [`crate::Bundle::from_fs`].
#[derive(Debug)]
pub enum BundleFromFsError {
    /// Failed to build a [`crate::schemas::raw_locale_map::RawLocaleMap`].
    Read(RawLocaleMapError),

    /// Failed to build a [`crate::Bundle`].
    Transform(TransformError),
}

/// Errors of [`crate::schemas::raw_locale_map::RawLocaleMap::from_fs`].
#[derive(Debug)]
pub enum RawLocaleMapError {
    /// Failed to read a file.
    ReadFile(ReadFileError),

    /// Failed to parse a YAML file according to the schema.
    Yaml(YamlError),
}

/// See [`RawLocaleMapError::ReadFile`].
#[derive(Debug)]
pub struct ReadFileError {
    pub path: PathBuf,
    pub error: io::Error,
}

/// See [`RawLocaleMapError::Yaml`].
#[derive(Debug)]
pub struct YamlError {
    pub inner: Box<serde_saphyr::Error>,
    pub filename: PathBuf,
    pub source_code: String,
}

/// Errors of [`crate::schemas::transform`].
#[derive(Debug)]
pub enum TransformError {
    /// A key was not parsed successfully (wrong syntax).
    InvalidKey(InvalidKeyError),

    /// A [`crate::Template`] was not parsed succesfully (wrong syntax).
    InvalidTemplate(InvalidTemplateError),

    /// A key corresponding to a namespace in the main locale
    /// points to a message in another locale.
    NotANamespace(NotANamespaceError),

    /// A key corresponding to a message in the main locale
    /// points to a namespace in another locale.
    NotAMessage(NotAMessageError),

    /// A translation of a message has parameters not specified
    /// in the main translation of this message.
    UnknownParameters(UnknownParametersError),
}

/// See [`TransformError::InvalidKey`].
#[derive(Debug)]
pub struct InvalidKeyError {
    pub locale: Language,

    /// [`None`] if no parent exists (i.e., the root namespace's node).
    pub parent_key: Option<RawDottedKey>,

    pub errors: ChumskyAllErrors,
}

/// See [`TransformError::InvalidTemplate`].
#[derive(Debug)]
pub struct InvalidTemplateError {
    pub locale: Language,
    pub key: RawDottedKey,
    pub errors: ChumskyAllErrors,
}

/// See [`TransformError::NotANamespace`].
#[derive(Debug)]
pub struct NotANamespaceError {
    pub locale: Language,

    /// The misinterpreted key that should point to a namespace.
    pub key: RawDottedKey,
}

/// See [`TransformError::NotAMessage`].
#[derive(Debug)]
pub struct NotAMessageError {
    pub locale: Language,
    pub key: RawDottedKey,
}

/// See [`TransformError::UnknownParameters`].
#[derive(Debug)]
pub struct UnknownParametersError {
    pub locale: Language,
    pub key: RawDottedKey,
    pub parameters: BTreeSet1<Identifier>,
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
    pub message: String,

    /// The problematic byte indices in the `source` string.
    pub span: Range<usize>,
}

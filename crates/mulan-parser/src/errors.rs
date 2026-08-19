//! Error types.

use std::io;
use std::path::PathBuf;
use std::range::Range;

use compact_str::CompactString;
use mitsein::btree_set1::BTreeSet1;
use mitsein::small_vec1::SmallVec1;
use mulan_config::Language;

use crate::{Parameter, RawKey};

/// Errors of [`crate::compose`].
#[derive(Debug)]
pub enum ComposeError {
    /// Failed to build a [`crate::schemas::input::Input`].
    Read(InputError),

    /// Failed to build a [`crate::Output`].
    Transform(TransformError),
}

/// Errors of [`crate::schemas::input::Input::read`].
#[derive(Debug)]
pub enum InputError {
    /// Failed to read a file.
    ReadFile(ReadFileError),

    /// Failed to parse a YAML file according to the schema.
    Yaml(YamlError),
}

/// See [`InputError::ReadFile`].
#[derive(Debug)]
pub struct ReadFileError {
    pub path: PathBuf,
    pub error: io::Error,
}

/// See [`InputError::Yaml`].
#[derive(Debug)]
pub struct YamlError {
    pub inner: Box<serde_saphyr::Error>,
    pub filename: PathBuf,
    pub source_code: String,
}

/// Errors of [`crate::schemas::transform`].
#[derive(Debug)]
pub enum TransformError {
    /// A [`crate::Subkey`] was not parsed successfully (wrong syntax).
    InvalidSubkey(InvalidSubkeyError),

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

/// See [`TransformError::InvalidSubkey`].
#[derive(Debug)]
pub struct InvalidSubkeyError {
    pub locale: Language,

    /// [`None`] if no parent exists (i.e., the root namespace's node).
    pub parent_key: Option<RawKey>,

    pub errors: ChumskyAllErrors,
}

/// See [`TransformError::InvalidTemplate`].
#[derive(Debug)]
pub struct InvalidTemplateError {
    pub locale: Language,
    pub key: RawKey,
    pub errors: ChumskyAllErrors,
}

/// See [`TransformError::NotANamespace`].
#[derive(Debug)]
pub struct NotANamespaceError {
    pub locale: Language,

    /// The misinterpreted key that should point to a namespace.
    pub key: RawKey,
}

/// See [`TransformError::NotAMessage`].
#[derive(Debug)]
pub struct NotAMessageError {
    pub locale: Language,
    pub key: RawKey,
}

/// See [`TransformError::UnknownParameters`].
#[derive(Debug)]
pub struct UnknownParametersError {
    pub locale: Language,
    pub key: RawKey,
    pub parameters: BTreeSet1<Parameter>,
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

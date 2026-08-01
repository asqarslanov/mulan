//! Conversions from custom Mulan error types to human-readable reports that can
//! be displayed in the CLI.
//!
//! This module defines the [`ToReport`] trait and implements it for all used
//! error types.

use std::range::Range;

use compact_str::CompactString;
use mitsein::small_vec1::SmallVec1;

/// A trait to converting strongly typed errors to human-readable
/// [`miette::Report`]s with [`ToReport::to_report`].
///
/// This trait is implemented automatically for all types that implement
/// [`self::ReportData`].
///
/// ---
///
/// Typically, you want to implement [`ToReport`] manually
/// for types that act as branchers for underlying types that also implement
/// [`ToReport`].
///
/// ```ignore
/// impl ToReport for MyError {
///     fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
///         match self {
///             Self::Foo(e) => e.to_report(config),
///             Self::Bar(e) => e.to_report(config),
///         }
///     }
/// }
/// ```
pub trait ToReport {
    /// Converts this error to a [`miette::Report`], respecting the user's
    /// config.
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report;
}

/// ...
trait ReportData {
    /// ...
    fn message(&self, config: &mulan_config::Config) -> String;

    /// A globally unique diagnostic code in the Rust path format
    /// (e.g., `parser::validate::not_a_message`).
    fn code(&self) -> &'static str;

    /// A user-friendly message on what could be the cause of the error
    /// or how to fix it.
    fn help(&self, config: &mulan_config::Config) -> Option<String>;

    /// ...
    fn source_code_data(&self) -> Option<self::SourceCodeData>;
}

/// See [`ReportData::source_code_data`].
#[derive(Debug)]
struct SourceCodeData {
    source_code: String,

    /// ...
    file_data: Option<self::SourceCodeFileData>,

    /// ...
    labels: SmallVec1<[self::SourceCodeLabel; 1]>,
}

/// ...
#[derive(Debug)]
struct SourceCodeFileData {
    /// ...
    name: CompactString,

    /// ...
    language: &'static str,
}

/// ...
#[derive(Debug)]
struct SourceCodeLabel {
    /// ...
    text: Option<String>,

    /// ...
    span: self::LabelSpan,
}

/// ...
#[derive(Debug)]
enum LabelSpan {
    /// ...
    Index(usize),

    /// ...
    OffsetLen(usize, usize),

    /// ...
    Range(Range<usize>),

    /// ...
    Full,
}

impl<E: ReportData> ToReport for E {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        let (source_data, labels) = match self.source_code_data() {
            None => (None, None),
            Some(SourceCodeData {
                source_code,
                file_data,
                labels,
            }) => {
                let labeled_spans: Vec<miette::LabeledSpan> = {
                    labels
                        .into_iter()
                        .map(|label| {
                            let span: miette::SourceSpan = match label.span {
                                LabelSpan::Index(i) => i.into(),
                                LabelSpan::OffsetLen(offset, len) => {
                                    if len == 1 {
                                        offset.into()
                                    } else {
                                        (offset..offset + len).into()
                                    }
                                }
                                LabelSpan::Range(Range { start, end }) => {
                                    if start + 1 == end {
                                        start.into()
                                    } else {
                                        (start..end).into()
                                    }
                                }
                                LabelSpan::Full => (0..=source_code.len()).into(),
                            };
                            miette::LabeledSpan::new_with_span(label.text, span)
                        })
                        .collect()
                };
                (Some((source_code, file_data)), Some(labeled_spans))
            }
        };
        let mut report = miette::Report::from(miette::MietteDiagnostic {
            message: self.message(config),
            code: Some(self.code().to_owned()),
            severity: Some(miette::Severity::Error),
            help: self.help(config),
            url: None,
            labels,
        });
        if let Some((source_code, file_data)) = source_data {
            report = match file_data {
                Some(file) => report.with_source_code(
                    miette::NamedSource::new(file.name, source_code).with_language(file.language),
                ),
                None => report.with_source_code(source_code),
            };
        }
        report
    }
}

impl ToReport for mulan_config::errors::ConfigError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Figment(e) => e.to_report(config),
            Self::Meta(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_config::errors::FigmentError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        self.inner.to_string()
    }

    fn code(&self) -> &'static str {
        use figment2::error::Kind as K;
        match self.inner.kind {
            K::Message(_) => "config::parse::message",
            K::InvalidType(_, _) => "config::parse::invalid_type",
            K::InvalidValue(_, _) => "config::parse::invalid_value",
            K::InvalidLength(_, _) => "config::parse::invalid_length",
            K::UnknownVariant(_, _) => "config::parse::unknown_variant",
            K::UnknownField(_, _) => "config::parse::unknown_field",
            K::MissingField(_) => "config::parse::missing_field",
            K::DuplicateField(_) => "config::parse::duplicate_field",
            K::ISizeOutOfRange(_) => "config::parse::isize_out_of_range",
            K::USizeOutOfRange(_) => "config::parse::usize_out_of_range",
            K::Unsupported(_) => "config::parse::unsupported",
            K::UnsupportedKey(_, _) => "config::parse::unsupported_key",
        }
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        None
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }
}

impl ToReport for mulan_config::errors::MetaError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::CurrentDir(e) => e.to_report(config),
            Self::SourceNotFound(e) => e.to_report(config),
            Self::AmbiguousSource(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_config::errors::CurrentDirError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_config::errors::SourceNotFoundError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_config::errors::AmbiguousSourceError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ToReport for mulan_parser::errors::ComposeError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Read(e) => e.to_report(config),
            Self::Transform(e) => e.to_report(config),
        }
    }
}

impl ToReport for mulan_parser::errors::InputError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::ReadFile(e) => e.to_report(config),
            Self::Yaml(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_parser::errors::ReadFileError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::YamlError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        use serde_saphyr::Error as E;
        match self.inner.without_snippet() {
            E::DuplicateMappingKey { key, location: _ } => {
                format!(
                    "duplicate mapping key{}",
                    key.as_ref()
                        .map_or_else(String::default, |k| format!(": `{k}`")),
                )
            }
            e => e.render(),
        }
    }

    fn code(&self) -> &'static str {
        use serde_saphyr::Error as E;
        match self.inner.without_snippet() {
            E::Eof { .. } => "parser::read::yaml::eof",
            E::MultipleDocuments { .. } => "parser::read::yaml::multiple_documents",
            E::Unexpected { .. } => "parser::read::yaml::unexpected",
            E::MergeValueNotMapOrSeqOfMaps { .. } => {
                "parser::read::yaml::merge_value_not_map_or_seq_of_maps"
            }
            E::MergeKeyNotAllowed { .. } => "parser::read::yaml::merge_key_not_allowed",
            E::InvalidBinaryBase64 { .. } => "parser::read::yaml::invalid_binary_base64",
            E::BinaryNotUtf8 { .. } => "parser::read::yaml::binary_not_utf8",
            E::TaggedScalarCannotDeserializeIntoString { .. } => {
                "parser::read::yaml::tagged_scalar_cannot_deserialize_into_string"
            }
            E::UnexpectedSequenceEnd { .. } => "parser::read::yaml::unexpected_sequence_end",
            E::UnexpectedMappingEnd { .. } => "parser::read::yaml::unexpected_mapping_end",
            E::InvalidBooleanStrict { .. } => "parser::read::yaml::invalid_boolean_strict",
            E::InvalidCharNull { .. } => "parser::read::yaml::invalid_char_null",
            E::InvalidCharNotSingleScalar { .. } => {
                "parser::read::yaml::invalid_char_not_single_scalar"
            }
            E::NullIntoString { .. } => "parser::read::yaml::null_into_string",
            E::BytesNotSupportedMissingBinaryTag { .. } => {
                "parser::read::yaml::bytes_not_supported_missing_binary_tag"
            }
            E::UnexpectedValueForUnit { .. } => "parser::read::yaml::unexpected_value_for_unit",
            E::ExpectedEmptyMappingForUnitStruct { .. } => {
                "parser::read::yaml::expected_empty_mapping_for_unit_struct"
            }
            E::UnexpectedContainerEndWhileSkippingNode { .. } => {
                "parser::read::yaml::unexpected_container_end_while_skipping_node"
            }
            E::InternalSeedReusedForMapKey { .. } => {
                "parser::read::yaml::internal_seed_reused_for_map_key"
            }
            E::ValueRequestedBeforeKey { .. } => "parser::read::yaml::value_requested_before_key",
            E::ExpectedStringKeyForExternallyTaggedEnum { .. } => {
                "parser::read::yaml::expected_string_key_for_externally_tagged_enum"
            }
            E::ExternallyTaggedEnumExpectedScalarOrMapping { .. } => {
                "parser::read::yaml::externally_tagged_enum_expected_scalar_or_mapping"
            }
            E::UnexpectedValueForUnitEnumVariant { .. } => {
                "parser::read::yaml::unexpected_value_for_unit_enum_variant"
            }
            E::InvalidUtf8Input => "parser::read::yaml::invalid_utf8_input",
            E::AliasReplayCounterOverflow { .. } => {
                "parser::read::yaml::alias_replay_counter_overflow"
            }
            E::AliasReplayLimitExceeded { .. } => "parser::read::yaml::alias_replay_limit_exceeded",
            E::AliasExpansionLimitExceeded { .. } => {
                "parser::read::yaml::alias_expansion_limit_exceeded"
            }
            E::AliasReplayStackDepthExceeded { .. } => {
                "parser::read::yaml::alias_replay_stack_depth_exceeded"
            }
            E::FoldedBlockScalarMustIndentContent { .. } => {
                "parser::read::yaml::folded_block_scalar_must_indent_content"
            }
            E::InternalDepthUnderflow { .. } => "parser::read::yaml::internal_depth_underflow",
            E::InternalRecursionStackEmpty { .. } => {
                "parser::read::yaml::internal_recursion_stack_empty"
            }
            E::RecursiveReferencesRequireWeakTypes { .. } => {
                "parser::read::yaml::recursive_references_require_weak_types"
            }
            E::InvalidScalar { .. } => "parser::read::yaml::invalid_scalar",
            E::SerdeInvalidType { .. } => "parser::read::yaml::serde_invalid_type",
            E::SerdeInvalidValue { .. } => "parser::read::yaml::serde_invalid_value",
            E::SerdeUnknownVariant { .. } => "parser::read::yaml::serde_unknown_variant",
            E::SerdeUnknownField { .. } => "parser::read::yaml::serde_unknown_field",
            E::SerdeMissingField { .. } => "parser::read::yaml::serde_missing_field",
            E::UnexpectedContainerEndWhileReadingKeyNode { .. } => {
                "parser::read::yaml::unexpected_container_end_while_reading_key_node"
            }
            E::DuplicateMappingKey { .. } => "parser::read::yaml::duplicate_mapping_key",
            E::TaggedEnumMismatch { .. } => "parser::read::yaml::tagged_enum_mismatch",
            E::SerdeVariantId { .. } => "parser::read::yaml::serde_variant_id",
            E::ExpectedMappingEndAfterEnumVariantValue { .. } => {
                "parser::read::yaml::expected_mapping_end_after_enum_variant_value"
            }
            E::ContainerEndMismatch { .. } => "parser::read::yaml::container_end_mismatch",
            E::UnknownAnchor { .. } => "parser::read::yaml::unknown_anchor",
            E::CyclicInclude { .. } => "parser::read::yaml::cyclic_include",
            E::UnsupportedIncludeForm { .. } => "parser::read::yaml::unsupported_include_form",
            E::ResolverError { .. } => "parser::read::yaml::resolver_error",
            E::AliasError { .. } => "parser::read::yaml::alias_error",
            E::HookError { .. } => "parser::read::yaml::hook_error",
            E::UnresolvedProperty { .. } => "parser::read::yaml::unresolved_property",
            E::InvalidPropertyName { .. } => "parser::read::yaml::invalid_property_name",
            E::PropertyRequiredButUnset { .. } => "parser::read::yaml::property_required_but_unset",
            E::PropertyRequiredButEmpty { .. } => "parser::read::yaml::property_required_but_empty",
            E::Budget { .. } => "parser::read::yaml::budget",
            E::IOError { .. } => "parser::read::yaml::io_error",
            E::QuotingRequired { .. } => "parser::read::yaml::quoting_required",
            E::CannotBorrowTransformedString { .. } => {
                "parser::read::yaml::cannot_borrow_transformed_string"
            }
            E::IndentationError { .. } => "parser::read::yaml::indentation_error",
            _ => "parser::read::yaml",
        }
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        use serde_saphyr::Error as E;
        match self.inner.without_snippet() {
            E::DuplicateMappingKey {
                key: _,
                location: _,
            } => Some("remove duplicates to make all keys in the same namespace unique".to_owned()),
            _ => None,
        }
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        let span = self.inner.location()?.span();
        let offset = usize::try_from(span.offset()).ok()?;
        let len = usize::try_from(span.len()).ok()?;
        Some(self::SourceCodeData {
            source_code: self.source_code.clone(),
            file_data: Some(SourceCodeFileData {
                name: self.filename.to_string_lossy().into(),
                language: "YAML",
            }),
            labels: SmallVec1::from_one(SourceCodeLabel {
                text: None,
                span: LabelSpan::OffsetLen(offset, len),
            }),
        })
    }
}

impl ToReport for mulan_parser::errors::TransformError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::MissingLocale(e) => e.to_report(config),
            Self::InvalidSubkey(e) => e.to_report(config),
            Self::InvalidTemplate(e) => e.to_report(config),
            Self::NotANamespace(e) => e.to_report(config),
            Self::NotAMessage(e) => e.to_report(config),
            Self::UnknownParameters(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_parser::errors::MissingLocaleError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::InvalidSubkeyError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::InvalidTemplateError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::NotANamespaceError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::NotAMessageError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::UnknownParametersError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

impl ToReport for mulan_parser::errors::ChumskyAllErrors {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("...")]
        #[diagnostic(code("..."), help("..."))]
        struct Helper(#[related] SmallVec1<[miette::Report; 1]>);

        todo!();

        let related_errors = {
            self.errors
                .iter1()
                .map(|error| {
                    ChumskyErrorWrapper {
                        error,
                        source: &self.source,
                    }
                    .to_report(config)
                })
                .collect1()
        };
        Helper(related_errors).into()
    }
}

/// ...
struct ChumskyErrorWrapper<'err> {
    error: &'err mulan_parser::errors::ChumskySingleError,

    /// ...
    source: &'err str,
}

impl ReportData for self::ChumskyErrorWrapper<'_> {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        todo!()
    }
}

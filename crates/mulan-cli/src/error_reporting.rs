use miette::{LabeledSpan, MietteDiagnostic};

/// ...
pub trait ToReport {
    /// ...
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report;
}

impl<E: ReportData> ToReport for E {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        let source_code_data = self.source_code_data();
        MietteDiagnostic {
            message: self.message(config),
            code: Some(self.code().to_owned()),
            severity: Some(miette::Severity::Error),
            help: self.help(config),
            url: None,
            labels: source_code_data.map(|data| data.labels),
        }
        .into()
    }
}

struct SourceCodeData {
    source_code: String,
    language: Option<&'static str>,
    labels: Vec<LabeledSpan>,
}

/// ...
trait ReportData {
    /// ...
    fn message(&self, config: &mulan_config::Config) -> String;

    /// ...
    fn code(&self) -> &'static str;

    /// ...
    fn help(&self, config: &mulan_config::Config) -> Option<String>;

    /// ...
    fn source_code_data(&self) -> Option<SourceCodeData>;
}

impl ToReport for mulan_config::errors::ConfigError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Figment(e) => e.to_report(config),
            Self::Meta(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_config::errors::FigmentError {}

impl ToReport for mulan_config::errors::MetaError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::CurrentDir(e) => e.to_report(config),
            Self::SourceNotFound(e) => e.to_report(config),
            Self::AmbiguousSource(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_config::errors::CurrentDirError {}

impl ReportData for mulan_config::errors::SourceNotFoundError {}

impl ReportData for mulan_config::errors::AmbiguousSourceError {}

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

impl ReportData for mulan_parser::errors::ReadFileError {}

impl ReportData for mulan_parser::errors::YamlError {}

impl ToReport for mulan_parser::errors::TransformError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::LocaleNotFound(e) => e.to_report(config),
            Self::InvalidSubkey(e) => e.to_report(config),
            Self::InvalidTemplate(e) => e.to_report(config),
            Self::NotANamespace(e) => e.to_report(config),
            Self::NotAMessage(e) => e.to_report(config),
            Self::UnknownParameters(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_parser::errors::LocaleNotFoundError {}

impl ReportData for mulan_parser::errors::InvalidSubkeyError {}

impl ReportData for mulan_parser::errors::InvalidTemplateError {}

impl ReportData for mulan_parser::errors::NotANamespaceError {}

impl ReportData for mulan_parser::errors::NotAMessageError {}

impl ReportData for mulan_parser::errors::UnknownParametersError {}

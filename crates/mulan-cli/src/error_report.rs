use miette::{LabeledSpan, MietteDiagnostic};

/// ...
pub trait ToReport {
    /// ...
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report;
}

impl<E: ReportData> ToReport for E {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        MietteDiagnostic {
            message: self.message(config),
            code: Some(self.code().to_owned()),
            severity: Some(miette::Severity::Error),
            help: self.help(config),
            url: None,
            labels: self.labels(config),
        }
        .into()
    }
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
    fn source_code(&self) -> Option<String>;

    /// ...
    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>>;
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
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
        todo!()
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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
        "parser::input::read_file"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::YamlError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        "parser::input::yaml"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
        todo!()
    }
}

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

impl ReportData for mulan_parser::errors::LocaleNotFoundError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        todo!()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
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

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
        todo!()
    }
}

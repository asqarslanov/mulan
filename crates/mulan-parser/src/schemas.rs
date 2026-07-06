//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`].

use std::collections::{BTreeMap, BTreeSet};

use compact_str::CompactString;
use mitsein::small_vec1::SmallVec1;
use mitsein::vec1::Vec1;
use mulan_config::Language;
use smallvec::{SmallVec, ToSmallVec as _};

pub use self::input::Input; // TODO: use it privately
use self::input::{DefinitionAtError, RawNamespace, RawNode};
use self::output::Output;
use crate::chumsky_parse::{ChumskyAllErrors, ChumskyParser};
use crate::{Namespace, Node, Subkey, Template, Translations};

mod input;
pub mod output;

/// Errors of [`transform`].
#[derive(Debug)]
enum TransformError {
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
}

/// ...
fn transform(
    mut input: Input,
    subkey_parser: &impl for<'src> ChumskyParser<'src, Subkey>,
    template_parser: &impl for<'src> ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Output, TransformError> {
    let default_locale = {
        input
            .locales
            .remove(&config.default_locale)
            .ok_or(TransformError::LocaleNotFound(config.default_locale))?
    };
    let namespace = traverse_namespace(
        SmallVec::default(),
        default_locale.root,
        &input,
        subkey_parser,
        template_parser,
        config,
    )?;
    Ok(Output { root: todo!() })
}

/// ...
fn traverse_namespace(
    key: SmallVec<[CompactString; 1]>,
    namespace: RawNamespace,
    input: &Input,
    subkey_parser: &impl for<'src> ChumskyParser<'src, Subkey>,
    template_parser: &impl for<'src> ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Namespace, TransformError> {
    let map = {
        namespace
            .map
            .into_iter()
            .map(|(raw_subkey, raw_node)| {
                let subkey = subkey_parser.mulan_parse(&raw_subkey).map_err(|errors| {
                    TransformError::InvalidSubkey {
                        locale: config.default_locale,
                        path: key.to_owned(),
                        errors,
                    }
                })?;
                let key = SmallVec1::from_rtail_and_head(key.clone(), raw_subkey);
                let handle_node_result =
                    handle_node(raw_node, key, input, subkey_parser, template_parser, config);
                let node = handle_node_result?;
                Ok((subkey, node))
            })
            .collect::<Result<_, _>>()?
    };
    Ok(Namespace { map })
}

/// ...
fn handle_node(
    raw_node: RawNode,
    key: SmallVec1<[CompactString; 1]>,
    input: &Input,
    subkey_parser: &impl for<'src> ChumskyParser<'src, Subkey>,
    template_parser: &impl for<'src> ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Node, TransformError> {
    let node = match raw_node {
        RawNode::Message(raw_template) => {
            let template = {
                template_parser
                    .mulan_parse(&raw_template)
                    .map_err(|errors| TransformError::InvalidTemplate {
                        locale: config.default_locale,
                        key: key.to_owned(),
                        errors,
                    })?
            };
            Node::Message(translations(input, key, template, template_parser, config)?)
        }
        RawNode::Namespace(inner_namespace) => Node::Namespace(traverse_namespace(
            key.into(),
            inner_namespace,
            input,
            subkey_parser,
            template_parser,
            config,
        )?),
    };
    Ok(node)
}

/// ...
fn translations<'src>(
    input: &'src Input,
    key: SmallVec1<[CompactString; 1]>,
    default: Template,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Translations, TransformError> {
    let others = {
        config
            .locales_except_default()
            .filter_map(|locale| {
                let Some(definition) = input.locales.get(&locale) else {
                    return Some(Err(TransformError::LocaleNotFound(locale)));
                };
                let node = match definition.at(&key) {
                    Ok(node) => node,
                    Err(DefinitionAtError::NotFound { index: _ }) => return None,
                    Err(DefinitionAtError::NotANamespace { index }) => {
                        return Some(Err(TransformError::NotANamespace {
                            locale,
                            key: key.to_vec1(),
                            index,
                        }));
                    }
                };
                let Some(raw_template) = node.try_as_message_ref() else {
                    return Some(Err(TransformError::NotAMessage {
                        locale,
                        key: key.clone(),
                    }));
                };
                let template = match template_parser.mulan_parse(raw_template) {
                    Ok(template) => template,
                    Err(errors) => {
                        return Some(Err(TransformError::InvalidTemplate {
                            locale,
                            key: key.clone(),
                            errors,
                        }));
                    }
                };
                let params = template.parameters().collect::<BTreeSet<_>>();
                Some(Ok((locale, template)))
            })
            .collect::<Result<_, _>>()?
    };
    Ok(Translations { default, others })
}

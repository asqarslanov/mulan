//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`].

use std::collections::BTreeMap;

use compact_str::CompactString;
use foldhash::HashSet;
use mitsein::btree_set1::BTreeSet1;
use mitsein::iter1::IteratorExt as _;
use mitsein::slice1::Slice1;
use mitsein::small_vec1::SmallVec1;
use mitsein::vec1::Vec1;
use mulan_config::Language;
use smallvec::SmallVec;

use self::input::{Definition, DefinitionAtError, Input, RawNamespace, RawNode};
use self::output::Output;
use crate::chumsky_parse::{ChumskyAllErrors, ChumskyParser};
use crate::{Namespace, Node, Parameter, Subkey, Template, Translations};

pub mod input;
pub mod output;

/// Errors of [`transform`].
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
pub fn transform<'src>(
    input: &'src Input,
    default_locale: &'src Definition,
    subkey_parser: &impl ChumskyParser<'src, Subkey>,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Output, TransformError> {
    let root = traverse_namespace(
        &[],
        &default_locale.root,
        input,
        subkey_parser,
        template_parser,
        config,
    )?;
    Ok(Output { root })
}

/// ...
fn traverse_namespace<'src>(
    key: &[CompactString],
    namespace: &'src RawNamespace,
    input: &'src Input,
    subkey_parser: &impl ChumskyParser<'src, Subkey>,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Namespace, TransformError> {
    let mut map = BTreeMap::new();
    for (raw_subkey, raw_node) in &namespace.map {
        let subkey = subkey_parser.mulan_parse(raw_subkey).map_err(|errors| {
            TransformError::InvalidSubkey {
                locale: config.default_locale,
                path: key.into(),
                errors,
            }
        })?;
        let key = SmallVec1::<[CompactString; 1]>::from_rtail_and_head(
            key.iter().cloned(),
            raw_subkey.clone(),
        );
        let handle_node_result = handle_node(
            raw_node,
            &key,
            input,
            subkey_parser,
            template_parser,
            config,
        );
        let node = handle_node_result?;
        map.insert(subkey, node);
    }
    Ok(Namespace { map })
}

/// ...
fn handle_node<'src>(
    raw_node: &'src RawNode,
    key: &Slice1<CompactString>,
    input: &'src Input,
    subkey_parser: &impl ChumskyParser<'src, Subkey>,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Node, TransformError> {
    let node = match raw_node {
        RawNode::Message(raw_template) => {
            let template = {
                template_parser
                    .mulan_parse(raw_template)
                    .map_err(|errors| TransformError::InvalidTemplate {
                        locale: config.default_locale,
                        key: key.into(),
                        errors,
                    })?
            };
            Node::Message(translations(input, key, template, template_parser, config)?)
        }
        RawNode::Namespace(inner_namespace) => Node::Namespace(traverse_namespace(
            key.as_slice(),
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
    key: &Slice1<CompactString>,
    default: Template,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Translations, TransformError> {
    let default_params: HashSet<_> = default.parameters().collect();
    let mut others = BTreeMap::new();
    for locale in config.locales_except_default() {
        let Some(definition) = input.locales.get(&locale) else {
            return Err(TransformError::LocaleNotFound(locale));
        };
        let node = match definition.at(key) {
            Ok(node) => node,
            Err(DefinitionAtError::NotFound { index: _ }) => continue,
            Err(DefinitionAtError::NotANamespace { index }) => {
                return Err(TransformError::NotANamespace {
                    locale,
                    key: key.into(),
                    index,
                });
            }
        };
        let Some(raw_template) = node.try_as_message_ref() else {
            return Err(TransformError::NotAMessage {
                locale,
                key: key.into(),
            });
        };
        let template = match template_parser.mulan_parse(raw_template) {
            Ok(template) => template,
            Err(errors) => {
                return Err(TransformError::InvalidTemplate {
                    locale,
                    key: key.into(),
                    errors,
                });
            }
        };
        let params = template.parameters().collect::<HashSet<_>>();
        let unknown_params = params.difference(&default_params).copied();
        if let Ok(unknown_params) = unknown_params.try_into_iter1() {
            return Err(TransformError::UnknownParameters {
                locale,
                key: key.into(),
                parameters: unknown_params.cloned().collect1(),
            });
        }
        others.insert(locale, template);
    }
    Ok(Translations { default, others })
}

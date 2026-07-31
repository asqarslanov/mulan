//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`]
//! (see the [`transform`] function).

use std::collections::BTreeMap;

use compact_str::CompactString;
use foldhash::HashSet;
use mitsein::iter1::IteratorExt as _;
use mitsein::slice1::Slice1;
use mitsein::vec1::Vec1;

use self::input::{Definition, DefinitionAtError, Input, RawNamespace, RawNode};
use self::output::Output;
use crate::chumsky_parse::ChumskyParser;
use crate::errors::TransformError;
use crate::{Namespace, Node, Subkey, Template, Translations};

pub mod input;
pub mod output;

/// Tries to transform an [`Input`] to a validated [`Output`].
pub fn transform<'src>(
    input: &'src Input,
    main_locale: &'src Definition,
    subkey_parser: &impl ChumskyParser<'src, Subkey>,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Output, TransformError> {
    let root = traverse_namespace(
        &[],
        &main_locale.root,
        input,
        subkey_parser,
        template_parser,
        config,
    )?;
    Ok(Output { root })
}

/// Recursively goes over a [`RawNamespace`] of the main locale,
/// collects corresponding nodes from other locales, and combines
/// everything into a proper [`Namespace`].
fn traverse_namespace<'src>(
    key: &[&str],
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
                locale: config.main_locale,
                path: key.iter().map(CompactString::new).collect(),
                errors,
            }
        })?;
        let key = Vec1::<&str>::from_rtail_and_head(key.iter().copied(), raw_subkey);
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
    key: &Slice1<&str>,
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
                        locale: config.main_locale,
                        key: key.iter1().map(CompactString::new).collect1(),
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
    key: &Slice1<&str>,
    main: Template,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Translations, TransformError> {
    let main_params: HashSet<_> = main.parameters().collect();
    let mut others = BTreeMap::new();
    for locale in config.locales_except_main() {
        let Some(definition) = input.locales.get(&locale) else {
            return Err(TransformError::LocaleNotFound(locale));
        };
        let node = match definition.at(key) {
            Ok(node) => node,
            Err(DefinitionAtError::NotFound { index: _ }) => {
                // If a locale doesn't have a message that exists in the main locale,
                // we just skip this message. The main locale will later act as a fallback.
                continue;
            }
            Err(DefinitionAtError::NotANamespace { index }) => {
                return Err(TransformError::NotANamespace {
                    locale,
                    key: key.iter1().map(CompactString::new).collect1(),
                    index,
                });
            }
        };
        let Some(raw_template) = node.try_as_message_ref() else {
            return Err(TransformError::NotAMessage {
                locale,
                key: key.iter1().map(CompactString::new).collect1(),
            });
        };
        let template = match template_parser.mulan_parse(raw_template) {
            Ok(template) => template,
            Err(errors) => {
                return Err(TransformError::InvalidTemplate {
                    locale,
                    key: key.iter1().map(CompactString::new).collect1(),
                    errors,
                });
            }
        };
        let params = template.parameters().collect::<HashSet<_>>();
        let unknown_params = params.difference(&main_params).copied();
        if let Ok(unknown_params) = unknown_params.try_into_iter1() {
            return Err(TransformError::UnknownParameters {
                locale,
                key: key.iter1().map(CompactString::new).collect1(),
                parameters: unknown_params.cloned().collect1(),
            });
        }
        others.insert(locale, template);
    }
    Ok(Translations { main, others })
}

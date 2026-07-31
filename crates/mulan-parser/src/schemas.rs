//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`].

use std::collections::BTreeMap;

use compact_str::CompactString;
use foldhash::HashSet;
use mitsein::iter1::IteratorExt as _;
use mitsein::slice1::Slice1;
use mitsein::small_vec1::SmallVec1;

use self::input::{Definition, DefinitionAtError, Input, RawNamespace, RawNode};
use self::output::Output;
use crate::chumsky_parse::ChumskyParser;
use crate::errors::ComposeError;
use crate::{Namespace, Node, Subkey, Template, Translations};

pub mod input;
pub mod output;

/// ...
pub fn transform<'src>(
    input: &'src Input,
    main_locale: &'src Definition,
    subkey_parser: &impl ChumskyParser<'src, Subkey>,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Output, ComposeError> {
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

/// ...
fn traverse_namespace<'src>(
    key: &[CompactString],
    namespace: &'src RawNamespace,
    input: &'src Input,
    subkey_parser: &impl ChumskyParser<'src, Subkey>,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Namespace, ComposeError> {
    let mut map = BTreeMap::new();
    for (raw_subkey, raw_node) in &namespace.map {
        let subkey = subkey_parser.mulan_parse(raw_subkey).map_err(|errors| {
            ComposeError::InvalidSubkey {
                locale: config.main_locale,
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
) -> Result<Node, ComposeError> {
    let node = match raw_node {
        RawNode::Message(raw_template) => {
            let template = {
                template_parser
                    .mulan_parse(raw_template)
                    .map_err(|errors| ComposeError::InvalidTemplate {
                        locale: config.main_locale,
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
    main: Template,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Translations, ComposeError> {
    let main_params: HashSet<_> = main.parameters().collect();
    let mut others = BTreeMap::new();
    for locale in config.locales_except_main() {
        let Some(definition) = input.locales.get(&locale) else {
            return Err(ComposeError::LocaleNotFound(locale));
        };
        let node = match definition.at(key) {
            Ok(node) => node,
            Err(DefinitionAtError::NotFound { index: _ }) => {
                // If a locale doesn't have a message that exists in the main locale,
                // we just skip this message. The main locale will later act as a fallback.
                continue;
            }
            Err(DefinitionAtError::NotANamespace { index }) => {
                return Err(ComposeError::NotANamespace {
                    locale,
                    key: key.into(),
                    index,
                });
            }
        };
        let Some(raw_template) = node.try_as_message_ref() else {
            return Err(ComposeError::NotAMessage {
                locale,
                key: key.into(),
            });
        };
        let template = match template_parser.mulan_parse(raw_template) {
            Ok(template) => template,
            Err(errors) => {
                return Err(ComposeError::InvalidTemplate {
                    locale,
                    key: key.into(),
                    errors,
                });
            }
        };
        let params = template.parameters().collect::<HashSet<_>>();
        let unknown_params = params.difference(&main_params).copied();
        if let Ok(unknown_params) = unknown_params.try_into_iter1() {
            return Err(ComposeError::UnknownParameters {
                locale,
                key: key.into(),
                parameters: unknown_params.cloned().collect1(),
            });
        }
        others.insert(locale, template);
    }
    Ok(Translations { main, others })
}

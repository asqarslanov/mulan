//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`]
//! (see the [`transform`] function).

use std::collections::BTreeMap;

use foldhash::HashSet;
use mitsein::iter1::IteratorExt as _;
use mitsein::vec1::Vec1;

use self::input::{Definition, DefinitionAtError, Input, RawKey, RawNamespace, RawNode};
use self::output::{Namespace, Node, Output, Subkey, Translations};
use crate::chumsky_parse::ChumskyParser;
use crate::errors::TransformError;
use crate::{Parameter, Template};

pub mod input;
pub mod output;

/// Tries to transform an [`Input`] to a validated [`Output`].
pub fn transform<'input>(
    input: &'input Input,
    main_locale: &'input Definition,
    subkey_parser: &impl ChumskyParser<'input, Subkey>,
    template_parser: &impl ChumskyParser<'input, Template>,
    config: &mulan_config::Config,
) -> Result<Output, TransformError> {
    let root = traverse_namespace(
        None,
        &main_locale.root,
        input,
        subkey_parser,
        template_parser,
        config,
    )?;
    Ok(Output { root })
}

/// A brancher that, given a [`RawNode`] from the main locale,
/// either processes it as a message ([`translations`])
/// or as a namespace ([`traverse_namespace`]) to get a proper [`Node`].
fn handle_node<'input>(
    raw_node: &'input RawNode,
    key: &RawKey,
    input: &'input Input,
    subkey_parser: &impl ChumskyParser<'input, Subkey>,
    template_parser: &impl ChumskyParser<'input, Template>,
    config: &mulan_config::Config,
) -> Result<Node, TransformError> {
    let node = match raw_node {
        RawNode::Message(raw_template) => {
            let template = {
                template_parser
                    .mulan_parse(raw_template)
                    .map_err(|errors| TransformError::InvalidTemplate {
                        locale: config.main_locale,
                        key: key.clone(),
                        errors,
                    })?
            };
            Node::Message(translations(input, key, template, template_parser, config)?)
        }
        RawNode::Namespace(inner_namespace) => Node::Namespace(traverse_namespace(
            Some(key),
            inner_namespace,
            input,
            subkey_parser,
            template_parser,
            config,
        )?),
    };
    Ok(node)
}

/// Given a [`Template`] from the main locale, collects its counterparts from
/// other locales and builds a proper instance of [`Translations`].
fn translations<'input>(
    input: &'input Input,
    key: &RawKey,
    main_translation: Template,
    template_parser: &impl ChumskyParser<'input, Template>,
    config: &mulan_config::Config,
) -> Result<Translations, TransformError> {
    let main_params: HashSet<&Parameter> = main_translation.parameters().collect();
    let mut other_translations = BTreeMap::new();
    for locale in config.locales_except_main() {
        let Some(definition) = input.locales.get(&locale) else {
            return Err(TransformError::LocaleNotFound(locale));
        };
        let raw_node = match definition.at(key) {
            Ok(node) => node,
            Err(e) => match e {
                DefinitionAtError::NotFound { index: _ } => {
                    // If a locale doesn't have a message that exists in the main locale,
                    // we just skip this message. The main locale will later act as a fallback.
                    continue;
                }
                DefinitionAtError::NotANamespace { index } => {
                    let segments = Vec1::try_from(&key.segments[..=index])
                        .expect("`..=n` slices are always non-empty");
                    let key = RawKey { segments };
                    return Err(TransformError::NotANamespace { locale, key });
                }
            },
        };
        let Some(raw_template) = raw_node.try_as_message_ref() else {
            return Err(TransformError::NotAMessage {
                locale,
                key: key.clone(),
            });
        };
        let template = match template_parser.mulan_parse(raw_template) {
            Ok(template) => template,
            Err(errors) => {
                return Err(TransformError::InvalidTemplate {
                    locale,
                    key: key.clone(),
                    errors,
                });
            }
        };
        let params = template.parameters().collect::<HashSet<_>>();
        let unknown_params = params.difference(&main_params).copied();
        if let Ok(unknown_params) = unknown_params.try_into_iter1() {
            return Err(TransformError::UnknownParameters {
                locale,
                key: key.clone(),
                parameters: unknown_params.cloned().collect1(),
            });
        }
        other_translations.insert(locale, template);
    }
    Ok(Translations {
        main: main_translation,
        others: other_translations,
    })
}

/// Recursively goes over a [`RawNamespace`] of the main locale,
/// collects corresponding nodes from other locales, and combines
/// everything into a proper [`Namespace`].
///
/// If traversing the root namespace, set `namespace_key` to [`None`].
fn traverse_namespace<'input>(
    namespace_key: Option<&RawKey>,
    namespace: &'input RawNamespace,
    input: &'input Input,
    subkey_parser: &impl ChumskyParser<'input, Subkey>,
    template_parser: &impl ChumskyParser<'input, Template>,
    config: &mulan_config::Config,
) -> Result<Namespace, TransformError> {
    let mut map = BTreeMap::new();
    for (raw_subkey, raw_node) in &namespace.map {
        let subkey = subkey_parser.mulan_parse(raw_subkey).map_err(|errors| {
            TransformError::InvalidSubkey {
                locale: config.main_locale,
                parent_key: namespace_key.cloned(),
                errors,
            }
        })?;
        let rtail = {
            namespace_key
                .map(|key| key.segments.to_vec())
                .unwrap_or_default()
        };
        let key = RawKey {
            segments: Vec1::from_rtail_and_head(
                rtail,
                raw_subkey.clone(), // after obtaining `subkey`, we're sure `raw_subkey` is valid
            ),
        };
        let node = handle_node(
            raw_node,
            &key,
            input,
            subkey_parser,
            template_parser,
            config,
        )?;
        map.insert(subkey, node);
    }
    Ok(Namespace { map })
}

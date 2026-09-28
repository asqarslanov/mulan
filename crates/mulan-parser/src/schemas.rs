//! The parent module of [`mod@locale_map`] and [`mod@bundle`].
//!
//! Defines the transformation logic from [`LocaleMap`] to [`Bundle`]
//! (see the [`transpose`] function).

use std::collections::BTreeMap;

use foldhash::HashSet;
use mitsein::iter1::IteratorExt as _;
use mitsein::vec1::Vec1;

use self::bundle::{Bundle, Namespace, Node, Translations};
use self::locale_map::{LDefinition, LDefinitionAtError, LNamespace, LNode, LocaleMap};
use crate::errors::{NotAMessageError, NotANamespaceError, TransposeError, UnknownParametersError};
use crate::{DottedKey, Identifier, Template};

pub mod bundle;
pub mod locale_map;

/// Tries to transform a [`LocaleMap`] to a [`Bundle`].
pub fn transpose<'input>(
    config: &mulan_config::Config,
    locale_map: &'input LocaleMap,
    main_locale: &'input LDefinition,
) -> Result<Bundle, TransposeError> {
    let root = traverse_namespace(config, None, &main_locale.root, locale_map)?;
    Ok(Bundle { root })
}

/// A brancher that, given a [`RawNode`] from the main locale,
/// either processes it as a message ([`translations`])
/// or as a namespace ([`traverse_namespace`]) to get a proper [`Node`].
fn handle_node<'input>(
    config: &mulan_config::Config,
    raw_node: &'input LNode,
    key: &DottedKey,
    locale_map: &'input LocaleMap,
) -> Result<Node, TransposeError> {
    let node = match raw_node {
        LNode::Message(l_template) => {
            Node::Message(translations(config, locale_map, key, l_template.clone())?)
        }
        LNode::Namespace(inner_namespace) => Node::Namespace(traverse_namespace(
            config,
            Some(key),
            inner_namespace,
            locale_map,
        )?),
    };
    Ok(node)
}

/// Given a [`Template`] from the main locale, collects its counterparts from
/// other locales and builds a proper instance of [`Translations`].
fn translations<'input>(
    config: &mulan_config::Config,
    locale_map: &'input LocaleMap,
    key: &DottedKey,
    main_translation: Template,
) -> Result<Translations, TransposeError> {
    let main_params: HashSet<&Identifier> = main_translation.parameter_iter().collect();
    let mut other_translations = BTreeMap::new();
    for locale in config.locales_except_main() {
        let definition = {
            locale_map
                .locales
                .get(&locale)
                .expect("all locales should've been read when parsing `input`")
        };
        let l_node = match definition.at(key) {
            Ok(node) => node,
            Err(e) => match e {
                LDefinitionAtError::NotFound { index: _ } => {
                    // If a locale doesn't have a message that exists
                    // in the main locale, we just skip this message.
                    // The main locale will later act as a fallback.
                    continue;
                }
                LDefinitionAtError::NotANamespace { index } => {
                    let segments = Vec1::try_from(&key.parts[..=index])
                        .expect("`..=n` slices are always non-empty");
                    let key = DottedKey { parts: segments };
                    let err = NotANamespaceError { locale, key };
                    return Err(TransposeError::NotANamespace(err));
                }
            },
        };
        let Some(template) = l_node.try_as_message_ref() else {
            let key = key.clone();
            let err = NotAMessageError { locale, key };
            return Err(TransposeError::NotAMessage(err));
        };
        let params = template.parameter_iter().collect::<HashSet<_>>();
        let unknown_params = params.difference(&main_params).copied();
        if let Ok(unknown_params) = unknown_params.try_into_iter1() {
            return Err(TransposeError::UnknownParameters(UnknownParametersError {
                locale,
                key: key.clone(),
                parameters: unknown_params.cloned().collect1(),
            }));
        }
        other_translations.insert(locale, template.clone());
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
    config: &mulan_config::Config,
    namespace_key: Option<&DottedKey>,
    namespace: &'input LNamespace,
    locale_map: &'input LocaleMap,
) -> Result<Namespace, TransposeError> {
    let mut map = BTreeMap::new();
    for (key_part, raw_node) in &namespace.map {
        let rtail = {
            namespace_key
                .map(|key| key.parts.to_vec())
                .unwrap_or_default()
        };
        let key = DottedKey {
            parts: Vec1::from_rtail_and_head(rtail, key_part.clone()),
        };
        let node = handle_node(config, raw_node, &key, locale_map)?;
        map.insert(key_part.clone(), node);
    }
    Ok(Namespace { map })
}

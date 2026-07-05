//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`].

use std::collections::{BTreeMap, BTreeSet};

use compact_str::CompactString;
use mitsein::small_vec1::SmallVec1;
use mulan_config::Language;
use smallvec::SmallVec;

pub use self::input::Input; // TODO: use it privately
use self::input::{DefinitionAtError, RawNamespace, RawNode};
use self::output::Output;
use crate::chumsky_parse::{ChumskyAllErrors, ChumskyParser};
use crate::{Namespace, Node, Subkey, Template};

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
        path: SmallVec1<[CompactString; 1]>,
        errors: ChumskyAllErrors,
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
                    handle_node(raw_node, key, subkey_parser, template_parser, config);
                let node = handle_node_result?;
                Ok((subkey, node))
            })
            .collect::<Result<_, _>>()?
    };
    Ok(Namespace { map })
}

/// ...
fn handle_node(
    node: RawNode,
    key: SmallVec1<[CompactString; 1]>,
    subkey_parser: &impl for<'src> ChumskyParser<'src, Subkey>,
    template_parser: &impl for<'src> ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Node, TransformError> {
    Ok(match node {
        RawNode::Message(template_raw) => {
            let template = {
                template_parser
                    .mulan_parse(&template_raw)
                    .map_err(|errors| TransformError::InvalidTemplate {
                        locale: config.default_locale,
                        path: key.to_owned(),
                        errors,
                    })?
            };
            todo!();
        }
        RawNode::Namespace(inner_namespace) => Node::Namespace(traverse_namespace(
            key.into(),
            inner_namespace,
            subkey_parser,
            template_parser,
            config,
        )?),
    })
}

/// ...
fn translations<'src>(
    input: &'src Input,
    path: &SmallVec1<[CompactString; 2]>,
    template_parser: &impl ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<BTreeMap<Language, Template>, ()> {
    config
        .locales_except_default()
        .filter_map(|lang| {
            let Some(definition) = input.locales.get(&lang) else {
                return Some(Err(()));
            };
            let node = match definition.at(path) {
                Ok(node) => node,
                Err(DefinitionAtError::NotFound { index: _ }) => return None,
                Err(DefinitionAtError::NotANamespace { index: _ }) => return Some(Err(())),
            };
            let Some(raw_template) = node.try_as_message_ref() else {
                return Some(Err(()));
            };
            let template = match template_parser.mulan_parse(raw_template) {
                Ok(template) => template,
                Err(_err) => return Some(Err(())),
            };
            let params = template.parameters().collect::<BTreeSet<_>>();
            Some(Ok((lang, template)))
        })
        .collect()
}

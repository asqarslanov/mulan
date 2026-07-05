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
use crate::{Namespace, Node, Template};

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
        path: SmallVec1<[CompactString; 1]>,
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
        &mut SmallVec::new(),
        default_locale.root,
        template_parser,
        config,
    )?;
    Ok(Output { root: todo!() })
}

/// ...
fn traverse_namespace(
    key: &mut SmallVec<[CompactString; 1]>,
    namespace: RawNamespace,
    template_parser: &impl for<'src> ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Namespace, TransformError> {
    for (subkey, raw_node) in namespace.map {
        key.push(subkey);
        let handle_node_result = handle_node(raw_node, key, template_parser, config);
        key.pop();
        let node = handle_node_result?;
    }
    todo!();
}

/// ...
fn handle_node(
    node: RawNode,
    key: &mut SmallVec<[CompactString; 1]>,
    template_parser: &impl for<'src> ChumskyParser<'src, Template>,
    config: &mulan_config::Config,
) -> Result<Node, TransformError> {
    match node {
        RawNode::Message(template_raw) => {
            let a = template_parser
                .mulan_parse(&template_raw)
                .map_err(|errors| TransformError::InvalidTemplate {
                    locale: config.default_locale,
                    path: {
                        key.to_owned()
                            .try_into()
                            .expect("`subkey` pushed at every iteration")
                    },
                    errors,
                })?;
        }
        RawNode::Namespace(inner_namespace) => {
            traverse_namespace(key, inner_namespace, template_parser, config)?;
        }
    }
    todo!();
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

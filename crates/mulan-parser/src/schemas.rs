//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`].

use std::collections::{BTreeMap, BTreeSet};

use compact_str::CompactString;
use mitsein::small_vec1::SmallVec1;
use mulan_config::Language;

pub use self::input::Input; // TODO: use it privately
use self::input::{DefinitionAtError, RawNode};
use self::output::Output;
use crate::Template;
use crate::chumsky_parse::ChumskyParser;

mod input;
pub mod output;

/// ...
#[derive(Debug)]
enum TransformError {
    /// ...
    NoDefaultLocale,
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
            .ok_or(TransformError::NoDefaultLocale)?
    };
    for (subkey, node) in default_locale.root.map {
        match node {
            RawNode::Message(template_raw) => {
                let template = template_parser.mulan_parse(&template_raw);
                todo!();
            }
            RawNode::Namespace(namespace_raw) => todo!(),
        }
    }
    Ok(Output { root: todo!() })
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

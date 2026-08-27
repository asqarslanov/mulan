//! Defines the [`LocaleMap`].

use foldhash::HashMap;
use mulan_config::Language;

use crate::{Identifier, Template};

///
#[derive(Debug)]
pub struct LocaleMap {
    ///
    locales: HashMap<Language, LmDefinition>,
}

///
#[derive(Debug)]
struct LmDefinition {
    ///
    root: LmNamespace,
}

///
#[derive(Debug)]
struct LmNamespace {
    ///
    map: HashMap<Identifier, LmNode>,
}

///
#[derive(Debug)]
enum LmNode {
    ///
    Message(Template),

    ///
    Namespace(LmNamespace),
}

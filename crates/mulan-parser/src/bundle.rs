use std::collections::BTreeMap;

use mulan_config::Language;

use crate::identifier::Identifier;

///
#[derive(Debug)]
pub struct Bundle {
    ///
    pub root: BNamespace,
}

///
#[derive(Debug)]
pub struct BNamespace {
    ///
    pub(super) map: BTreeMap<Identifier, BNode>,
}

///
#[derive(Debug)]
pub enum BNode {
    ///
    Message(BMessage),

    ///
    Namespace(BNamespace),
}

///
#[derive(Debug)]
pub struct BMessage {
    ///
    pub main: Template,

    ///
    pub others: BTreeMap<Language, Template>,
}

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
enum Parameter {
    Text(Box<str>),
    Object {
        name: Box<str>,
        #[serde(default)]
        data_type: Type,
    },
}

#[derive(Deserialize)]
struct Options {
    emoji: bool,
    character_entity: bool,
    prettify: bool,
}

#[derive(Deserialize)]
struct FullSyntax {
    #[serde(alias = "params", default)]
    parameters: Box<[Parameter]>,
    #[serde(alias = "txt")]
    text: Box<str>,
    #[serde(alias = "opts")]
    options: Options,
}

struct Lowercase(Box<str>);

struct Identifier(Box<[Lowercase]>);

#[derive(Default, Deserialize)]
enum Type {
    #[default]
    Str,
    Int,
    Float,
}

struct Variable {
    name: Identifier,
    data_type: Type,
}

enum Token {
    Text(Box<str>),
    Variable(Variable),
    Extract {
        name: Identifier,
        contents: Box<Token>,
    },
}

pub struct Template(Box<[Token]>);

enum Element {
    Key(Template),
    Section(Box<[Element]>),
}

struct Locale(HashMap<Identifier, Element>);

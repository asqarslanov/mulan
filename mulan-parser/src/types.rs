use std::collections::HashMap;

struct Lowercase(Box<str>);

struct Identifier(Box<[Lowercase]>);

enum DataType {
    Str(Box<str>),
    Int(i32),
    Float(f64),
}

struct Variable {
    name: Identifier,
    data_type: DataType,
}

enum Token {
    Text(Box<str>),
    Variable(Variable),
    Extract {
        name: Identifier,
        contents: Box<Token>,
    },
}

struct Template(Box<[Token]>);

enum Element {
    Key(Template),
    Section(Box<[Element]>),
}

struct Locale(HashMap<Identifier, Element>);

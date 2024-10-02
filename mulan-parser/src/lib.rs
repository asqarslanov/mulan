enum DataType {
    Str(Box<str>),
    Int(i32),
    Float(f64),
}

struct Variable {
    name: Box<str>,
    data_type: DataType,
}

enum Token {
    Text(Box<str>),
    Variable(Variable),
    Extract {
        name: Box<str>,
        contents: Box<Token>,
    },
}

struct Template(Box<[Token]>);

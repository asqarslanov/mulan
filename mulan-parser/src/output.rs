use std::collections::HashMap;

#[derive(Debug)]
pub struct Template(Box<str>);

#[derive(Debug)]
pub enum Rhs {
    Text(Template),
    Section(Translation),
    Unimplemented,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Identifier(Box<[Box<str>]>);

#[derive(Debug)]
pub struct Translation(HashMap<Identifier, Rhs>);

impl From<crate::input::Translation> for Translation {
    fn from(value: crate::input::Translation) -> Self {
        Self(
            value
                .into_iter()
                .map(|(name, rhs)| {
                    let identifier = convert_name(name);
                    let rhs = convert_rhs(rhs, &identifier);
                    (identifier, rhs)
                })
                .collect(),
        )
    }
}

fn convert_name(value: crate::input::Name) -> Identifier {
    // FIXME: convert from cases.
    Identifier(Box::new([value]))
}

fn convert_rhs(value: crate::input::Rhs, _identifier: &Identifier) -> Rhs {
    use crate::input::Rhs as R;
    match value {
        R::Text(s) => Rhs::Text(Template(s)),
        R::Expanded(obj) => Rhs::Text(Template(obj.text)),
        R::Bool(false) => Rhs::Unimplemented,
        R::Bool(true) => todo!("import short"),
        R::Import(..) => todo!("import"),
    }
}

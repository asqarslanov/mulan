use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::input;

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

impl TryFrom<input::Translation> for Translation {
    type Error = anyhow::Error;

    fn try_from(value: input::Translation) -> Result<Self> {
        let new_hashmap: Result<_> = value
            .into_iter()
            .map(|(name, rhs)| {
                let identifier = convert_name(name);
                let rhs = convert_rhs(rhs, &identifier)?;
                Ok((identifier, rhs))
            })
            .collect();

        Ok(Self(new_hashmap?))
    }
}

fn convert_name(value: input::Name) -> Identifier {
    // FIXME: convert from cases.
    Identifier(Box::new([value]))
}

fn convert_rhs(value: input::Rhs, _identifier: &Identifier) -> Result<Rhs> {
    use crate::input::Rhs as R;
    Ok(match value {
        R::Text(s) => Rhs::Text(Template(s)),
        R::Expanded(obj) => Rhs::Text(Template(obj.text)),
        R::Bool(false) => Rhs::Unimplemented,
        R::Bool(true) => todo!("import short"),
        R::Import(props) => {
            use crate::input::ImportProps as P;
            match props {
                P::True => Rhs::Section(Translation(HashMap::from([(
                    Identifier(Box::from([Box::from("imported"), Box::from("true")])),
                    Rhs::Unimplemented,
                )]))),
                P::Path(p) => Rhs::Section(Translation(HashMap::from([(
                    Identifier(Box::from([
                        Box::from("imported"),
                        Box::from(
                            p.to_str()
                                .context("failed to convert the path to a string")?,
                        ),
                    ])),
                    Rhs::Unimplemented,
                )]))),
            }
        }
    })
}

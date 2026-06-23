//! See [`MessageTemplate`].

use compact_str::CompactString;
use smallvec::SmallVec;

use crate::identifier::Identifier;

/// ...
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    name: Identifier,
}

/// ...
#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    parts: SmallVec<[TemplatePart; 1]>,
}

/// ...
#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplatePart {
    /// ...
    Text(CompactString),

    /// ...
    Placeholder(Parameter),
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;
    use smallvec::SmallVec;

    use super::{Parameter, Template, TemplatePart};
    use crate::identifier::Identifier;

    impl TemplatePart {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            let text = {
                choice((just("{{").to('{'), just("}}").to('}'), none_of("{}")))
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .map(|it| Self::Text(CompactString::new(it)))
            };
            let expr = {
                Identifier::chumsky_parser()
                    .padded()
                    .delimited_by(just('{'), just('}'))
                    .map(|name| Self::Placeholder(Parameter { name }))
            };
            choice((text, expr))
        }
    }

    impl Template {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            TemplatePart::chumsky_parser()
                .repeated()
                .collect()
                .then_ignore(end())
                .map(|parts: Vec<_>| Self {
                    parts: SmallVec::from_vec(parts),
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser as _;
    use rstest::rstest;

    use self::PseudoTemplatePart::{Txt, Var};
    use super::*;

    enum PseudoTemplatePart {
        Txt(&'static str),
        Var(&'static str),
    }

    #[rstest]
    #[case("", Some([].as_slice()))]
    #[case("  ", Some([Txt("  ")].as_slice()))]
    #[case("a\nb", Some([Txt("a\nb")].as_slice()))]
    #[case("Hey{{", Some([Txt("Hey{")].as_slice()))]
    #[case("Hey}}", Some([Txt("Hey}")].as_slice()))]
    #[case("Hello, {name}!", Some([Txt("Hello, "), Var("name"), Txt("!")].as_slice()))]
    #[case(
        "I have {n} apples! {n}!",
        Some([Txt("I have "), Var("n"), Txt(" apples! "), Var("n"), Txt("!")].as_slice()))
    ]
    #[case("{{lorem-ipsum}}", Some([Txt("{lorem-ipsum}")].as_slice()))]
    #[case("{{{lorem-ipsum}}}", Some([Txt("{"), Var("lorem-ipsum"), Txt("}")].as_slice()))]
    #[case("{{{ lorem-ipsum  }}}", Some([Txt("{"), Var("lorem-ipsum"), Txt("}")].as_slice()))]
    #[case("{{{{lorem-ipsum}}}}", Some([Txt("{{lorem-ipsum}}")].as_slice()))]
    #[case("{{{{  lorem-ipsum   }}}}", Some([Txt("{{  lorem-ipsum   }}")].as_slice()))]
    #[case("{{{{{lorem-ipsum}}}}}", Some([Txt("{{"), Var("lorem-ipsum"), Txt("}}")].as_slice()))]
    #[case(
        "aaa{bbb}ccc{{ddd}}eee{{{  fff  }}}ggg{{{{hhh}}}}iii",
        Some(
            [
                Txt("aaa"),
                Var("bbb"),
                Txt("ccc{ddd}eee{"),
                Var("fff"),
                Txt("}ggg{{hhh}}iii")
            ]
            .as_slice(),
        ),
    )]
    #[case("{}", None)]
    #[case("{lorem_ipsum}", None)]
    #[case("{", None)]
    #[case("}", None)]
    #[case("he}y", None)]
    #[case("he{y", None)]
    #[case("{a", None)]
    #[case("a}", None)]
    #[case("{six seven}", None)]
    fn parse(#[case] input: &str, #[case] expected_output: Option<&[PseudoTemplatePart]>) {
        let msg_parser = Template::chumsky_parser();
        let ident_parser = Identifier::chumsky_parser();
        let actual_output = msg_parser.parse(input).into_output();
        let expected_output = expected_output.map(|raw_parts| Template {
            parts: {
                raw_parts
                    .iter()
                    .map(|part| match part {
                        Txt(it) => TemplatePart::Text(CompactString::new(it)),
                        Var(it) => TemplatePart::Placeholder(Parameter {
                            name: ident_parser.parse(it).unwrap(),
                        }),
                    })
                    .collect()
            },
        });
        assert_eq!(actual_output, expected_output);
    }
}

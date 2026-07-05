//! See [`Template`].

use compact_str::CompactString;
use smallvec::SmallVec;

use crate::identifier::Identifier;

/// A message template that consists of raw text and variable placeholders.
/// For example:
///
/// ```txt
/// Hello, {name}!
/// ```
///
/// This template can later be converted to different syntaxes.
/// For example (JavaScript):
///
/// ```js
/// `Hello, ${name}!`
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    parts: SmallVec<[TemplatePart; 1]>,
}

/// A part of a [`Template`].
#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplatePart {
    /// Plain text to be used verbatim.
    Text(CompactString),

    /// A stand-in for a variable (`{foo}`).
    Placeholder(Parameter),
}

/// A variable placeholder in a [`Template`] (`{foo}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    name: Identifier,
}

/// ...
mod parser {
    use chumsky::prelude::*;

    use super::{Parameter, Template, TemplatePart};
    use crate::chumsky_parse::ChumskyParser;
    use crate::identifier::Identifier;

    impl Template {
        /// Parses `Hello, {name}!` to `["Hello, ", #name, "!"]`.
        #[must_use]
        pub fn chumsky_parser<'src>() -> impl ChumskyParser<'src, Self> {
            TemplatePart::chumsky_parser()
                .repeated()
                .collect()
                .map(|parts| Self { parts })
        }
    }

    impl TemplatePart {
        /// Differentiates between different template part types.
        #[must_use]
        pub fn chumsky_parser<'src>() -> impl ChumskyParser<'src, Self> {
            let text = {
                choice((just("{{").to('{'), just("}}").to('}'), none_of("{}")))
                    .repeated()
                    .at_least(1)
                    .collect()
                    .map(Self::Text)
            };
            let placeholder = Parameter::chumsky_parser().map(Self::Placeholder);
            choice((text, placeholder))
        }
    }

    impl Parameter {
        /// Extracts `x` from `{x}`.
        #[must_use]
        pub fn chumsky_parser<'src>() -> impl ChumskyParser<'src, Self> {
            Identifier::chumsky_parser()
                .padded()
                .delimited_by(just('{'), just('}'))
                .map(|name| Self { name })
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use self::PseudoTemplatePart::{Txt, Var};
    use super::*;
    use crate::chumsky_parse::ChumskyParser as _;

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
        let actual_output = msg_parser.mulan_parse(input).ok();
        let expected_output = expected_output.map(|raw_parts| Template {
            parts: {
                raw_parts
                    .iter()
                    .map(|part| match part {
                        Txt(it) => TemplatePart::Text(CompactString::new(it)),
                        Var(it) => TemplatePart::Placeholder(Parameter {
                            name: ident_parser.mulan_parse(it).unwrap(),
                        }),
                    })
                    .collect()
            },
        });
        assert_eq!(actual_output, expected_output);
    }
}

//! See [`Template`].

use compact_str::CompactString;
use mitsein::compact_string1::CompactString1;
use smallvec::SmallVec;
use strum::EnumTryAs;

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

impl Template {
    /// ...
    pub fn parameters(&self) -> impl Iterator<Item = &Parameter> {
        self.parts
            .iter()
            .filter_map(TemplatePart::try_as_placeholder_ref)
    }
}

/// A part of a [`Template`].
#[derive(Debug, Clone, PartialEq, Eq, EnumTryAs)]
pub enum TemplatePart {
    /// Plain text to be used verbatim.
    Text(CompactString),

    /// A stand-in for a variable (`{foo}`).
    Placeholder(Parameter),
}

/// A variable placeholder in a [`Template`] (`{foo}`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parameter {
    name: Identifier,
}

impl Parameter {
    /// Converts this parameter to a kebab-case string (e.g., `first-name`).
    pub fn to_kebab_case(&self) -> CompactString1 {
        self.name.to_kebab_case()
    }
}

/// Defines parsers with [`mod@chumsky`].
mod parser {
    use chumsky::prelude::*;

    use super::{Parameter, Template, TemplatePart};
    use crate::chumsky_parse::ChumskyParser;
    use crate::identifier::Identifier;

    impl Template {
        /// Parses `Hello, {name}!` to `["Hello, ", #name, "!"]`.
        #[must_use]
        pub fn chumsky_parser<'src>(
            part_parser: &impl ChumskyParser<'src, TemplatePart>,
        ) -> impl ChumskyParser<'src, Self> {
            part_parser.repeated().collect().map(|parts| Self { parts })
        }
    }

    impl TemplatePart {
        /// Differentiates between different template part types.
        #[must_use]
        pub fn chumsky_parser<'src>(
            param_parser: &impl ChumskyParser<'src, Parameter>,
        ) -> impl ChumskyParser<'src, Self> {
            let text = {
                choice((just("{{").to('{'), just("}}").to('}'), none_of("{}")))
                    .repeated()
                    .at_least(1)
                    .collect()
                    .map(Self::Text)
            };
            let placeholder = param_parser.map(Self::Placeholder);
            choice((text, placeholder))
        }
    }

    impl Parameter {
        /// Extracts `x` from `{x}`.
        #[must_use]
        pub fn chumsky_parser<'src>(
            ident_parser: &impl ChumskyParser<'src, Identifier>,
        ) -> impl ChumskyParser<'src, Self> {
            ident_parser
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
    use crate::identifier::Word;

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
        let word_parser = Word::chumsky_parser();
        let ident_parser = Identifier::chumsky_parser(&word_parser);
        let param_parser = Parameter::chumsky_parser(&ident_parser);
        let msg_part_parser = TemplatePart::chumsky_parser(&param_parser);
        let msg_parser = Template::chumsky_parser(&msg_part_parser);
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

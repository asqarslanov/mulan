//!

use std::sync::LazyLock;

use aho_corasick::AhoCorasick;
use compact_str::CompactString;
use mitsein::compact_string1::CompactString1;
use smallvec::SmallVec;
use strum::EnumTryAs;

use crate::identifier::Identifier;

///
#[derive(Debug, PartialEq, Eq)]
pub struct TemplateBuf {
    parts: SmallVec<[TemplateBufPart; 1]>,
}

impl TemplateBuf {
    ///
    #[must_use]
    pub(super) fn preview(&self, config: &mulan_config::Config) -> Option<CompactString1> {
        static AC: LazyLock<AhoCorasick> = LazyLock::new(|| {
            AhoCorasick::new(["{", "}"]).expect("valid aho-corasick patterns and config")
        });
        let mut buffer = CompactString::default();
        for part in self.iter() {
            use crate::legacy::TemplateBufPart as P;
            match part {
                P::Text(text) => buffer.push_str(&AC.replace_all(text, &["{{", "}}"])),
                P::Tag(TagBuf::Parameter(name)) => buffer.push_str(&name.parameter_preview(config)),
            }
        }
        buffer.try_into().ok()
    }

    ///
    pub fn iter(&self) -> impl Iterator<Item = &TemplateBufPart> {
        self.parts.iter()
    }

    ///
    pub fn parameter_iter(&self) -> impl Iterator<Item = &Identifier> {
        self.parts
            .iter()
            .filter_map(TemplateBufPart::try_as_tag_ref)
            .filter_map(TagBuf::try_as_parameter_ref)
    }

    ///
    #[must_use]
    pub fn try_as_plain_text(&self) -> Option<&str> {
        match self.parts.as_slice() {
            [] => Some(<&str>::default()),
            [TemplateBufPart::Text(text)] => Some(text),
            _ => None,
        }
    }

    ///
    #[must_use]
    pub(super) fn max_consecutive_backticks(&self) -> usize {
        let mut count = 0;
        self.parts
            .iter()
            .filter_map(TemplateBufPart::try_as_text_ref)
            .for_each(|text| {
                let mut current_count = 0;
                for c in text.chars() {
                    if c == '`' {
                        current_count += 1;
                    } else {
                        count = count.max(current_count);
                        current_count = 0;
                    }
                }
                count = count.max(current_count);
            });
        count
    }
}

///
#[derive(Debug, Clone, PartialEq, Eq, EnumTryAs)]
pub enum TemplateBufPart {
    ///
    Text(CompactString),

    ///
    Tag(TagBuf),
}

///
#[derive(Debug, Clone, PartialEq, Eq, EnumTryAs)]
pub enum TagBuf {
    ///
    Parameter(Identifier),
}

/// Defines parsers with [`mod@chumsky`].
mod parser {
    use chumsky::prelude::*;

    use super::{TagBuf, TemplateBuf, TemplateBufPart};
    use crate::chumsky_parse::ChumskyParser;
    use crate::identifier::Identifier;

    impl TemplateBuf {
        /// Parses `Hello, {name}!` to `["Hello, ", #name, "!"]`.
        #[must_use]
        pub fn chumsky_parser<'src>(
            part_parser: &impl ChumskyParser<'src, TemplateBufPart>,
        ) -> impl ChumskyParser<'src, Self> {
            part_parser.repeated().collect().map(|parts| Self { parts })
        }
    }

    impl TemplateBufPart {
        /// Differentiates between different template part types.
        #[must_use]
        pub fn chumsky_parser<'src>(
            tag_parser: &impl ChumskyParser<'src, TagBuf>,
        ) -> impl ChumskyParser<'src, Self> {
            let text = {
                choice((just("{{").to('{'), just("}}").to('}'), none_of("{}")))
                    .repeated()
                    .at_least(1)
                    .collect()
                    .map(Self::Text)
            };
            let placeholder = tag_parser.map(Self::Tag);
            choice((text, placeholder))
        }
    }

    impl TagBuf {
        /// Extracts `x` from `{x}` and dfferentiates between different
        /// tag types.
        #[must_use]
        pub fn chumsky_parser<'src>(
            ident_parser: &impl ChumskyParser<'src, Identifier>,
        ) -> impl ChumskyParser<'src, Self> {
            ident_parser
                .padded()
                .delimited_by(just('{'), just('}'))
                .map(Self::Parameter)
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
        let tag_parser = TagBuf::chumsky_parser(&ident_parser);
        let msg_part_parser = TemplateBufPart::chumsky_parser(&tag_parser);
        let msg_parser = TemplateBuf::chumsky_parser(&msg_part_parser);
        let actual_output = msg_parser.mulan_parse(input).ok();
        let expected_output = expected_output.map(|raw_parts| TemplateBuf {
            parts: {
                raw_parts
                    .iter()
                    .map(|part| match part {
                        Txt(it) => TemplateBufPart::Text(CompactString::new(it)),
                        Var(it) => TemplateBufPart::Tag(TagBuf::Parameter(
                            ident_parser.mulan_parse(it).unwrap(),
                        )),
                    })
                    .collect()
            },
        });
        assert_eq!(actual_output, expected_output);
    }
}

//! See [`Template`].

use std::sync::LazyLock;

use aho_corasick::AhoCorasick;
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
    /// Returns what this message (approximately) looks like in the locale file.
    ///
    /// E.g., it can literally return a string such as `"Hello, {name}!"`.
    /// Or `None`, if the template is empty.
    #[must_use]
    pub(super) fn preview(&self, config: &mulan_config::Config) -> Option<CompactString1> {
        static AC: LazyLock<AhoCorasick> = LazyLock::new(|| {
            AhoCorasick::new(["{", "}"]).expect("valid aho-corasick patterns and config")
        });
        let mut buffer = CompactString::default();
        for part in self.iter() {
            use crate::TemplatePart as P;
            match part {
                P::Text(text) => buffer.push_str(&AC.replace_all(text, &["{{", "}}"])),
                P::Tag(Tag::Parameter(name)) => buffer.push_str(&name.parameter_preview(config)),
            }
        }
        buffer.try_into().ok()
    }

    /// An iterator over all parts in the order they appear in the message.
    pub fn iter(&self) -> impl Iterator<Item = &TemplatePart> {
        self.parts.iter()
    }

    /// An iterator over [`TemplatePart::Tag`]/[`Tag::Parameter`] parts
    /// in the order they appear in the message (so duplicates can be present).
    pub fn parameter_iter(&self) -> impl Iterator<Item = &Identifier> {
        self.parts
            .iter()
            .filter_map(TemplatePart::try_as_tag_ref)
            .filter_map(Tag::try_as_parameter_ref)
    }

    /// Returns a plain text string without dynamic parameters
    /// if this template can presented as such.
    #[must_use]
    pub fn try_as_plain_text(&self) -> Option<&str> {
        match self.parts.as_slice() {
            [] => Some(<&str>::default()),
            [TemplatePart::Text(text)] => Some(text),
            _ => None,
        }
    }

    /// How many consecutive backticks (`` ` ``) this template contains.
    ///
    /// Needed for [`crate::Translations::markdown_preview`].
    /// If the result is more than 3, you can't simply wrap this template
    /// in a code block with three backticks.
    #[must_use]
    pub(super) fn max_consecutive_backticks(&self) -> usize {
        let mut count = 0;
        self.parts
            .iter()
            .filter_map(TemplatePart::try_as_text_ref)
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

/// A part of a [`Template`].
#[derive(Debug, Clone, PartialEq, Eq, EnumTryAs)]
pub enum TemplatePart {
    /// Plain text to be used verbatim.
    Text(CompactString),

    /// See [`Tag`].
    Tag(Tag),
}

/// A special expression enclosed in `{` `}` (e.g., a [`Parameter`]).
#[derive(Debug, Clone, PartialEq, Eq, EnumTryAs)]
pub enum Tag {
    /// A stand-in for a variable (`{foo}`).
    Parameter(Identifier),
}

/// Defines parsers with [`mod@chumsky`].
mod parser {
    use chumsky::prelude::*;

    use super::{Tag, Template, TemplatePart};
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
            tag_parser: &impl ChumskyParser<'src, Tag>,
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

    impl Tag {
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
        let tag_parser = Tag::chumsky_parser(&ident_parser);
        let msg_part_parser = TemplatePart::chumsky_parser(&tag_parser);
        let msg_parser = Template::chumsky_parser(&msg_part_parser);
        let actual_output = msg_parser.mulan_parse(input).ok();
        let expected_output = expected_output.map(|raw_parts| Template {
            parts: {
                raw_parts
                    .iter()
                    .map(|part| match part {
                        Txt(it) => TemplatePart::Text(CompactString::new(it)),
                        Var(it) => {
                            TemplatePart::Tag(Tag::Parameter(ident_parser.mulan_parse(it).unwrap()))
                        }
                    })
                    .collect()
            },
        });
        assert_eq!(actual_output, expected_output);
    }
}

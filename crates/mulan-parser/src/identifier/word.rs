//! See [`Word`].

use compact_str::CompactString;

/// A part of an [`Identifier`](crate::identifier::Identifier).
///
/// For example, the identifier `student-bs23-id006` consists of three [`Word`]s:
/// `student`, `bs23`, and `id006`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    pub(super) inner: CompactString,
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;

    use super::Word;

    impl Word {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            let letter = one_of('a'..='z').labelled("small latin letter");
            let consecutive_letters = letter.clone().repeated().at_least(1);
            let consecutive_digits = text::digits(10);
            let letter_digit_mix = choice((consecutive_letters, consecutive_digits)).repeated();
            let word = letter.then(letter_digit_mix);
            word.to_slice().map(|it: &str| Self {
                inner: CompactString::new(it),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser as _;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("w", Ok)]
    #[case("manylettersofthelatinalphabet", Ok)]
    #[case("manylettersofthelatinalphabet2", Ok)]
    #[case("manydigits281348214378912347892", Ok)]
    #[case("onedigit1", Ok)]
    #[case("digitin1themiddle", Ok)]
    #[case("fn", Ok)]
    #[case("def", Ok)]
    #[case("gen", Ok)]
    #[case("for", Ok)]
    #[case("in", Ok)]
    #[case("if", Ok)]
    #[case("while", Ok)]
    #[case("not", Ok)]
    #[case("let", Ok)]
    #[case("", Err)]
    #[case(" ", Err)]
    #[case("spaceattheend ", Err)]
    #[case(" spaceatthebeginning", Err)]
    #[case("1", Err)]
    #[case("123", Err)]
    #[case("1digitatthebeginning", Err)]
    #[case("ALLCAPS", Err)]
    #[case("oneCapital", Err)]
    #[case("W", Err)]
    #[case("apostrophe'", Err)]
    #[case("diacriticñ", Err)]
    #[case("cyrillicж", Err)]
    #[case("dash-", Err)]
    #[case("underscore_", Err)]
    #[case("colon:", Err)]
    #[case("semi;", Err)]
    #[case("at@", Err)]
    #[case("amp&", Err)]
    #[case("star*", Err)]
    #[case("hash#", Err)]
    #[case("slash/", Err)]
    #[case("dot.", Err)]
    #[case("newline\n", Err)]
    fn parse(#[case] input: &str, #[case] expected_output: fn(()) -> Result<(), ()>) {
        let parser = Word::chumsky_parser();
        let parse_result = parser.parse(input).into_result();
        assert_eq!(parse_result.is_ok(), expected_output(()).is_ok());
        if let Ok(parsed_word) = parse_result {
            let input_word = Word {
                inner: CompactString::new(input),
            };
            assert_eq!(input_word, parsed_word);
        }
    }
}

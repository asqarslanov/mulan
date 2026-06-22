//! See [`Word`].

use compact_str::CompactString;

/// ...
///
/// ## Valid Examples
///
/// ...
///
/// ## Invalid Examples
///
/// ...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    pub(super) inner: CompactString,
}

mod parser {
    use chumsky::prelude::*;

    use super::*;

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
    #[case("a", Ok)]
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

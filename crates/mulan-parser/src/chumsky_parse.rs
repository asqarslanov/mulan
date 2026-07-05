//! ...

use std::range::Range;

use chumsky::prelude::*;
use compact_str::{CompactString, ToCompactString as _};
use mitsein::iter1::IteratorExt as _;
use mitsein::small_vec1::SmallVec1;

/// ...
#[derive(Debug)]
pub struct ChumskyAllErrors {
    /// ...
    pub source: CompactString,

    /// ...
    pub errors: SmallVec1<[ChumskySingleError; 1]>,
}

/// ...
#[derive(Debug)]
pub struct ChumskySingleError {
    /// ...
    pub message: CompactString,

    /// ...
    pub span: Range<usize>,
}

/// ...
pub trait ChumskyParser<'src, Out>:
    Parser<'src, &'src str, Out, extra::Err<Rich<'src, char>>>
{
    /// ...
    fn mulan_parse(&self, source: &'src str) -> Result<Out, ChumskyAllErrors> {
        let result = self.parse(source).into_result();
        result.map_err(|errors| ChumskyAllErrors {
            source: source.into(),
            errors: errors
                .into_iter()
                .try_into_iter1()
                .expect("if parsing failed, there should at least be one error")
                .map(|err| ChumskySingleError {
                    message: err.to_compact_string(),
                    span: err.span().into_range().into(),
                })
                .collect1(),
        })
    }
}

impl<'src, Out, T: Parser<'src, &'src str, Out, extra::Err<Rich<'src, char>>>>
    ChumskyParser<'src, Out> for T
{
}

//! See [`MessageTemplate`].

use compact_str::CompactString;
use smallvec::SmallVec;

use crate::identifier::Identifier;

/// ...
#[derive(Debug)]
pub struct Parameter {
    name: Identifier,
}

/// ...
#[derive(Debug)]
pub struct Template {
    parts: SmallVec<[TemlpatePart; 1]>,
}

/// ...
#[derive(Debug)]
enum TemlpatePart {
    /// ...
    Text(CompactString),

    /// ...
    Placeholder(Parameter),
}

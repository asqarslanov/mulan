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
pub struct MessageTemplate {
    parts: SmallVec<[MessagePart; 1]>,
}

/// ...
#[derive(Debug)]
enum MessagePart {
    /// ...
    Raw(CompactString),

    /// ...
    Placeholder(Parameter),
}

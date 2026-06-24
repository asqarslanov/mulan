//! Defines structures this crate operates on and operations on them.
//!
//! Most notably, [`Input`] and [`Output`].

use self::input::Input;

mod input;

#[derive(Debug)]
enum ReadError {}

impl Input {
    fn read() -> Result<Self, ReadError> {
        todo!();
    }
}

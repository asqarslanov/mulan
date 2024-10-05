use std::hash::Hash;

use self::errors::ContextInitError;
use super::options::Options;

mod errors {
    use std::hash::Hash;

    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ContextInitError<ContextId: Eq + Hash> {
        #[error("context with id `{0:?}` already exists")]
        AlreadyExists(ContextId),
    }
}

pub fn init<ContextId>(path: ContextId) -> Result<(), ContextInitError<ContextId>>
where
    ContextId: Eq + Hash,
{
    todo!()
}

pub fn init_with<ContextId>(
    path: ContextId,
    options: Options,
) -> Result<(), ContextInitError<ContextId>>
where
    ContextId: Eq + Hash,
{
    todo!()
}

pub fn get() {
    todo!()
}

pub fn get_or_init() {
    todo!()
}

pub fn get_or_init_with() {
    todo!()
}

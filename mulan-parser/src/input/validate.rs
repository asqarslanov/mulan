// TODO: add meaningful errors.

use serde::{Deserialize, Deserializer};

pub fn only_true<'de>(deserializer: impl Deserializer<'de>) -> Result<(), ()> {
    let parsed = bool::deserialize(deserializer).map_err(|_| ())?;

    if parsed {
        Ok(())
    } else {
        Err(())
    }
}

pub fn tag<'de, D, T>(deserializer: D, tag: &str) -> Result<T, ()>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let (parsed_tag, data) = <(Box<str>, T)>::deserialize(deserializer).map_err(|_| ())?;

    if *parsed_tag == *tag {
        Ok(data)
    } else {
        Err(())
    }
}

pub fn tag_import<'de>(deserializer: impl Deserializer<'de>) -> Result<super::ImportProps, ()> {
    tag(deserializer, "import")
}

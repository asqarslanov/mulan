use std::path::Path;

pub fn locales_dir() -> Box<Path> {
    Box::from(Path::new("locales/"))
}

pub fn entry_point() -> Box<str> {
    Box::from("locale")
}

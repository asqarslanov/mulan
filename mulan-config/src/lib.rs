use std::path::Path;

fn detect_config() -> Option<Box<Path>> {
    const POSSIBLE_CONFIG_FILE_LOCATIONS: [&str; 2] =
        [".mulan/mulan.toml", ".config/mulan/mulan.toml"];

    POSSIBLE_CONFIG_FILE_LOCATIONS
        .iter()
        .map(Path::new)
        .find(|path| path.exists())
        .map(Box::from)
}

use super::input::Input;
use super::output::Output;

#[derive(Debug)]
enum TransformError {
    NoDefaultLocale,
}

fn transform(mut input: Input, config: &mulan_config::Config) -> Result<Output, TransformError> {
    let default_locale = {
        input
            .locales
            .remove(&config.default_locale)
            .ok_or(TransformError::NoDefaultLocale)?
    };
    // for (key, node) in default_locale.root.map {}
    Ok(Output { root })
}

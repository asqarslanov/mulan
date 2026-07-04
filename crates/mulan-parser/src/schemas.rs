//! The parent module of [`mod@input`] and [`mod@output`].
//!
//! Defines the transformation logic from [`Input`] to [`Output`].

pub use self::input::Input; // TODO: use it privately
use self::output::Output;

mod input;
pub mod output;

/// ...
#[derive(Debug)]
enum TransformError {
    /// ...
    NoDefaultLocale,
}

/// ...
fn transform(mut input: Input, config: &mulan_config::Config) -> Result<Output, TransformError> {
    let default_locale = {
        input
            .locales
            .remove(&config.default_locale)
            .ok_or(TransformError::NoDefaultLocale)?
    };
    // for (key, node) in default_locale.root.map {}
    Ok(Output { root: todo!() })
}

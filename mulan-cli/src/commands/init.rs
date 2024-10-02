use anyhow::Result;
use inquire::Text;

pub fn init() -> Result<()> {
    let _x = Text::new("First")
        .with_help_message("Test")
        .with_placeholder("<DIRECTORY>")
        .with_initial_value("locales/")
        .prompt()?;

    Ok(())
}

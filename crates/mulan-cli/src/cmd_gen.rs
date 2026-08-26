use crate::error_reporting::ToReport as _;
use crate::i18n::{Locale, t};

/// Executes the subcommand:
///
/// ```sh
/// $ mulan gen ...
/// ```
pub fn execute() -> miette::Result<()> {
    let config = mulan_config::Config::locate_and_read()
        .map_err(|err| err.to_report(&mulan_config::Config::dummy()))?;
    let bundle = mulan_parser::Bundle::from_fs(&config).map_err(|err| err.to_report(&config))?;
    mulan_gen::write_files(&config, &bundle).map_err(|err| err.to_report(&config))?;
    let targets = config.generate.expect("mulan-gen verified they exist");
    for target in &targets {
        let log = match target {
            mulan_config::Target::Rust(target_conf) => t::cmd_gen::Log {
                path: target_conf.file.as_str(),
                target: "Rust",
            },
        };
        #[expect(clippy::print_stdout, reason = "deliberate logging")]
        {
            println!("{}", log.get_in(Locale::default()));
        }
    }
    Ok(())
}

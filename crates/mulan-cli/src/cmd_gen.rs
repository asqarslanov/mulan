use std::process::ExitCode;

use crate::error_reporting::ToReport as _;

#[derive(clap::Args)]
pub struct Args;

impl self::Args {
    /// Executes the subcommand:
    ///
    /// ```sh
    /// $ mulan gen ...
    /// ```
    #[expect(
        clippy::unused_self,
        reason = "to consistently use args.execute() in main"
    )]
    pub fn execute(self) -> miette::Result<ExitCode> {
        let config = mulan_config::Config::locate_and_read()
            .map_err(|err| err.to_report(&mulan_config::Config::dummy()))?;
        let output = mulan_parser::compose(&config).map_err(|err| err.to_report(&config))?;
        mulan_gen::write_files(&config, &output).map_err(|err| err.to_report(&config))?;
        let targets = config.generate.expect("mulan-gen verified they exist");
        for target in targets {
            match target {
                mulan_config::Target::Rust(target_conf) => {
                    println!("Generated Rust bindings in {path}", path = target_conf.file);
                }
            }
        }
        Ok(ExitCode::SUCCESS)
    }
}

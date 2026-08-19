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
        let Some(targets) = &config.generate else {
            todo!();
        };
        let output = mulan_parser::compose(&config).map_err(|err| err.to_report(&config))?;
        for target in targets {
            use mulan_config::Target as T;
            match target {
                T::Rust(_target_config) => {
                    let rust_bindings = mulan_gen::rust::generate(&config, &output);
                    println!("{rust_bindings}");
                }
            }
        }
        Ok(ExitCode::SUCCESS)
    }
}

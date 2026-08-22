use std::process::ExitCode;

use crate::error_reporting::ToReport as _;
use crate::i18n::Locale;
use crate::i18n::t;

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
        for target in &targets {
            let log = match target {
                mulan_config::Target::Rust(target_conf) => t::cmd_gen::Log {
                    path: target_conf.file.as_str(),
                    target: "Rust",
                },
            };
            #[expect(clippy::print_stdout, reason = "useful logging")]
            {
                println!("{}", log.get_in(Locale::default()));
            }
        }
        Ok(ExitCode::SUCCESS)
    }
}

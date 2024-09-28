use anyhow::Result;
use clap::Parser;
use pik::args::CliArgs;
use pik::processes::FilterOptions;
use pik::tui::start_app;

fn main() -> Result<()> {
    let mut config = pik::config::load_app_config()?;
    let args = CliArgs::parse();

    config.override_with_args(&args);
    start_app(
        args.query,
        //TODO: pass config instead
        FilterOptions {
            ignore_threads: true,
            include_all_processes: false,
        },
        20,
        false,
    )
}

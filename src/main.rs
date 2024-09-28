use anyhow::Result;
use clap::Parser;
use pik::args::CliArgs;
use pik::tui::start_app;

fn main() -> Result<()> {
    let mut config = pik::config::load_app_config()?;
    let args = CliArgs::parse();

    config.override_with_args(&args);
    start_app(args.query, config)
}

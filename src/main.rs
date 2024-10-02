use anyhow::Result;
use clap::Parser;
use pik::args::CliArgs;
use pik::settings::AppSettings;
use pik::tui::start_app;

fn main() -> Result<()> {
    let config = pik::config::load_app_config()?;
    let args = CliArgs::parse();

    let settings = AppSettings::from(config, &args);
    start_app(args.query, settings)
}

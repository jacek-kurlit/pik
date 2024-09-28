use anyhow::Result;
use clap::Parser;
use pik::args::CliArgs;
use pik::processes::FilterOptions;
use pik::tui::start_app;

fn main() -> Result<()> {
    let _ = pik::config::load_app_config()?;
    let args = CliArgs::parse();

    start_app(
        args.query,
        FilterOptions {
            ignore_threads: !args.include_threads_processes,
            include_all_processes: args.all_processes,
        },
        args.screen_size.height,
        args.screen_size.fullscreen,
    )
}

use anyhow::Result;
use clap::Parser;
use pik::args::Args;
use pik::processes::FilterOptions;
use pik::tui::start_app;

fn main() -> Result<()> {
    let _ = pik::config::load_app_config()?;
    let args = Args::parse();

    start_app(
        args.query,
        FilterOptions {
            ignore_threads: !args.include_threads_processes,
            include_all_processes: args.all_processes,
        },
        args.height,
        args.fullscreen,
    )
}

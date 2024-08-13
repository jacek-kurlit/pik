use anyhow::Result;
use args::Args;
use clap::Parser;
use processes::FilterOptions;
use tui::start_app;

mod args;
mod processes;
mod tui;

fn main() -> Result<()> {
    let args = Args::parse();
    start_app(
        args.query,
        FilterOptions {
            ignore_threads: !args.threads_processes,
            include_all_processes: args.all_processes,
        },
    )
}

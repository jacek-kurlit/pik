use anyhow::Result;
use args::Args;
use clap::Parser;
use tui::start_app;

mod args;
mod processes;
mod tui;

fn main() -> Result<()> {
    let args = Args::parse();
    start_app(args.query)
}

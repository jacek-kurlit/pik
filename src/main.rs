use std::env;

use anyhow::Result;
use tui::start_app;

mod processes;
mod tui;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    start_app(args.into_iter().next().unwrap_or("".to_string()))
}

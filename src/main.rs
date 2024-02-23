use std::env;

use anyhow::Result;
use tui::start_tui_app;

mod processes;
mod tui;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    start_tui_app(args.first().cloned().unwrap_or("".to_string()))
}

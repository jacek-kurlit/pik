use anyhow::Result;
use tui::start_tui_app;

mod processes;
mod tui;

fn main() -> Result<()> {
    start_tui_app()
}

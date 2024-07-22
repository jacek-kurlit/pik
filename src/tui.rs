use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, TerminalOptions, Viewport};

mod rendering;

use crate::processes::{ProcessManager, ProcessSearchResults};

use self::rendering::Tui;

struct App {
    process_manager: ProcessManager,
    search_results: ProcessSearchResults,
    tui: Tui,
}

impl App {
    fn new(search_criteria: String) -> Result<App> {
        let mut app = App {
            process_manager: ProcessManager::new()?,
            search_results: ProcessSearchResults::empty(),
            tui: Tui::new(search_criteria),
        };
        app.search_for_processess();
        Ok(app)
    }

    fn enter_char(&mut self, new_char: char) {
        self.tui.enter_char(new_char);
        self.search_for_processess();
    }

    fn search_for_processess(&mut self) {
        self.search_results = self
            .process_manager
            .find_processes(self.tui.search_input_text());
        self.tui.update_number_of_items(self.search_results.len());
    }

    fn delete_char(&mut self) {
        self.tui.delete_char();
        self.search_for_processess();
    }

    fn kill_selected_process(&mut self) {
        let prc_index = self.tui.get_selected_row_index();
        if let Some(prc) = self.search_results.nth(prc_index) {
            if self.process_manager.kill_process(prc.pid) {
                self.search_results.remove(prc_index);
                self.tui.update_number_of_items(self.search_results.len());
            }
        }
    }
}

pub fn start_app(search_criteria: String) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(20),
        },
    )?;

    // create app and run it
    let app = App::new(search_criteria)?;
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    terminal.clear()?;

    //FIXME: add error handling, for exaple some error page should be shown
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.tui.render_ui(&app.search_results, f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                use KeyCode::*;
                match key.code {
                    Esc => return Ok(()),
                    Up | BackTab => app.tui.select_previous_row(),
                    Tab | Down => app.tui.select_next_row(),
                    End => app.tui.move_search_cursor_to_end(),
                    Home => app.tui.move_search_cursor_to_start(),
                    Left => app.tui.move_search_cursor_left(),
                    Right => app.tui.move_search_cursor_right(),
                    Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.kill_selected_process()
                    }
                    Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.process_details_down()
                    }
                    Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.process_details_up()
                    }
                    Char(to_insert) => app.enter_char(to_insert),
                    Backspace => app.delete_char(),
                    _ => {}
                }
            }
        }
    }
}

use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, TerminalOptions};

mod rendering;

use crate::{
    processes::{FilterOptions, ProcessManager, ProcessSearchResults},
    settings::AppSettings,
};

use self::rendering::Tui;

struct App {
    process_manager: ProcessManager,
    search_results: ProcessSearchResults,
    filter_options: FilterOptions,
    tui: Tui,
}

impl App {
    fn new(search_criteria: String, app_settings: AppSettings) -> Result<App> {
        let mut app = App {
            process_manager: ProcessManager::new()?,
            search_results: ProcessSearchResults::empty(),
            filter_options: app_settings.filter_opions,
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
        self.tui.reset_error_message();
        self.process_manager.refresh();
        self.search_results = self
            .process_manager
            .find_processes(self.tui.search_input_text(), self.filter_options);
        self.tui
            .update_process_table_number_of_items(self.search_results.len());
    }

    fn delete_char(&mut self) {
        self.tui.delete_char();
        self.search_for_processess();
    }

    fn kill_selected_process(&mut self) {
        self.tui.reset_error_message();
        let prc_index = self.tui.get_selected_row_index();
        if let Some(prc) = self.search_results.nth(prc_index) {
            let pid = prc.pid;
            if self.process_manager.kill_process(pid) {
                self.search_for_processess();
                //NOTE: cache refresh takes time and process may reappear in list!
                self.search_results.remove(pid);
                //TODO: this must be here because details will show 1/0 when removed!
                // seems like this can only be fixed by autorefresh!
                self.tui
                    .update_process_table_number_of_items(self.search_results.len());
            } else {
                self.tui
                    .set_error_message("Failed to kill process, check permissions");
            }
        }
    }
}

pub fn start_app(search_criteria: String, app_settings: AppSettings) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let viewport = app_settings.viewport.clone();
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;

    // create app and run it
    let app = App::new(search_criteria, app_settings)?;
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
                    Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.select_first_row()
                    }
                    Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.select_last_row()
                    }
                    Up | BackTab => app.tui.select_previous_row(1),
                    Tab | Down => app.tui.select_next_row(1),
                    Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.select_next_row(1);
                    }
                    Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.select_previous_row(1);
                    }
                    PageUp => app.tui.select_previous_row(10),
                    PageDown => app.tui.select_next_row(10),
                    Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.kill_selected_process()
                    }
                    Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.search_for_processess()
                    }
                    Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.process_details_down(&mut terminal.get_frame())
                    }
                    Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.process_details_up()
                    }
                    Char(to_insert) => app.enter_char(to_insert),
                    Backspace => app.delete_char(),
                    _ => app.tui.handle_input(key),
                }
            }
        }
    }
}

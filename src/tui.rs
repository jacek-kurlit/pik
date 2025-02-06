use std::{io, rc::Rc};

use anyhow::Result;
use components::{
    help_footer::HelpFooterComponent, search_bar::SearchBarComponent, Action, Component,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, TerminalOptions};

pub mod components;
mod highlight;
mod rendering;

use crate::{
    processes::{IgnoreOptions, ProcessManager, ProcessSearchResults},
    settings::AppSettings,
};

use self::rendering::Tui;

struct App {
    process_manager: ProcessManager,
    search_results: ProcessSearchResults,
    ignore_options: IgnoreOptions,
    tui: Tui,
    search_bar: SearchBarComponent,
    healp_footer: HelpFooterComponent,
}

impl App {
    fn new(app_settings: AppSettings) -> Result<App> {
        let mut app = App {
            process_manager: ProcessManager::new()?,
            search_results: ProcessSearchResults::empty(),
            ignore_options: app_settings.filter_opions,
            tui: Tui::new(app_settings.use_icons),
            search_bar: SearchBarComponent::new(app_settings.query),
            healp_footer: HelpFooterComponent::default(),
        };
        app.search_for_processess();
        Ok(app)
    }

    pub fn enforce_search_by(&mut self, search_by: ProcessRelatedSearch) {
        let selected_index = self.tui.get_selected_row_index();
        let selected_process = self.search_results.nth(selected_index);
        if selected_process.is_none() {
            return;
        }
        let selected_process = selected_process.unwrap();
        let search_string = match search_by {
            ProcessRelatedSearch::Parent => {
                format!("!{}", selected_process.parent_pid.unwrap_or(0))
            }
            ProcessRelatedSearch::Family => {
                format!("@{}", selected_process.pid)
            }
            ProcessRelatedSearch::Siblings => {
                format!("@{}", selected_process.parent_pid.unwrap_or(0))
            }
        };
        //TODO: should be event
        self.search_bar.set_search_text(search_string);
        self.search_for_processess();
    }

    //TODO: this should not be here
    fn handle_event(&mut self, event: KeyEvent) {
        let action = self.search_bar.handle_input(event);
        if let Action::SearchForProcesses(_) = action {
            self.search_for_processess()
        }
    }

    fn search_for_processess(&mut self) {
        self.healp_footer.reset_error_message();
        self.process_manager.refresh();
        self.search_results = self
            .process_manager
            //TODO: refactor
            .find_processes(self.search_bar.get_search_text(), &self.ignore_options);
        self.tui
            .update_process_table_number_of_items(self.search_results.len());
    }

    fn kill_selected_process(&mut self) {
        self.healp_footer.reset_error_message();
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
                self.healp_footer
                    .set_error_message("Failed to kill process, check permissions");
            }
        }
    }
}

pub enum ProcessRelatedSearch {
    Family,   // process + process childrens
    Siblings, // process parent + all his children
    Parent,   // only parent process
}

pub fn start_app(app_settings: AppSettings) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let viewport = app_settings.viewport.clone();
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;

    // create app and run it
    let app = App::new(app_settings)?;
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
        terminal.draw(|f| {
            let rects = layout_rects(f);
            app.search_bar.render(f, rects[0]);
            app.healp_footer.render(f, rects[3]);
            app.tui.render_ui(&app.search_results, f, rects);
        })?;

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
                    PageUp => app.tui.select_previous_row(10),
                    PageDown => app.tui.select_next_row(10),
                    Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.select_next_row(1);
                    }
                    Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.tui.select_previous_row(1);
                    }
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
                    Char('p') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.enforce_search_by(ProcessRelatedSearch::Parent);
                    }
                    Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.enforce_search_by(ProcessRelatedSearch::Family);
                    }
                    Char('s') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.enforce_search_by(ProcessRelatedSearch::Siblings);
                    }
                    _ => app.handle_event(key),
                }
            }
        }
    }
}

fn layout_rects(frame: &mut Frame) -> Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Max(7),
        Constraint::Length(1),
    ])
    .split(frame.area())
}

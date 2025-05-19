use anyhow::Result;
use arboard::Clipboard;
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::CursorMove;

use crate::config::keymappings::AppAction;
use crate::{
    config::ui::UIConfig,
    processes::{IgnoreOptions, Process, ProcessManager, ProcessSearchResults},
    tui::{ProcessRelatedSearch, components::KeyAction},
};

use super::{
    Component, ComponentEvent, process_details::ProcessDetailsComponent,
    process_table::ProcessTableComponent, search_bar::SearchBarComponent,
};

pub struct ProcessesViewComponent {
    process_manager: ProcessManager,
    ignore_options: IgnoreOptions,
    search_results: ProcessSearchResults,
    process_table_component: ProcessTableComponent,
    process_details_component: ProcessDetailsComponent,
    search_bar: SearchBarComponent,
    clipboard: Clipboard,
}

impl ProcessesViewComponent {
    pub fn new(
        ui_config: &UIConfig,
        ignore_options: IgnoreOptions,
        initial_query: String,
    ) -> Result<Self> {
        let mut component = Self {
            process_manager: ProcessManager::new()?,
            ignore_options,
            search_results: ProcessSearchResults::empty(),
            process_table_component: ProcessTableComponent::new(
                ui_config.icons.get_icons(),
                // cloning for sake of simplicity
                ui_config.process_table.clone(),
            ),
            //TODO: fix this cloning later
            process_details_component: ProcessDetailsComponent::new(
                ui_config.process_details.clone(),
            ),
            search_bar: SearchBarComponent::new(
                initial_query,
                &ui_config.search_bar,
                ui_config.icons.get_icons().search_prompt.as_str(),
            ),
            clipboard: Clipboard::new()?,
        };
        component.search_for_processess();
        Ok(component)
    }

    pub fn select_first_row(&mut self) {
        self.process_table_component
            .select_first_row(self.search_results.len());
        self.process_details_component.reset_details_scroll_offset();
    }

    pub fn select_last_row(&mut self) {
        self.process_table_component
            .select_last_row(self.search_results.len());
        self.process_details_component.reset_details_scroll_offset();
    }

    pub fn select_next_row(&mut self, step_size: usize) {
        self.process_table_component
            .select_next_row(step_size, self.search_results.len());
        self.process_details_component.reset_details_scroll_offset();
    }

    pub fn select_previous_row(&mut self, step_size: usize) {
        self.process_table_component
            .select_previous_row(step_size, self.search_results.len());
        self.process_details_component.reset_details_scroll_offset();
    }

    fn get_selected_process(&self) -> Option<&Process> {
        let selected_index = self.process_table_component.get_selected_process_index();
        self.search_results.nth(selected_index)
    }

    fn update_process_table_state(&mut self) {
        let number_of_items = self.search_results.len();
        self.process_table_component
            .update_process_table_state(number_of_items);
    }

    fn search_for_processess(&mut self) -> KeyAction {
        let search_text = self.search_bar.get_search_text();
        self.process_manager.refresh();
        self.search_results = self
            .process_manager
            .find_processes(search_text, &self.ignore_options);
        self.update_process_table_state();
        KeyAction::Event(ComponentEvent::ProcessListRefreshed)
    }

    fn kill_selected_process(&mut self) -> KeyAction {
        if let Some(prc) = self.get_selected_process() {
            let pid = prc.pid;
            if self.process_manager.kill_process(pid) {
                self.search_for_processess();
                //NOTE: cache refresh takes time and process may reappear in list!
                self.search_results.remove(pid);
                //TODO: this must be here because details will show 1/0 when removed!
                // seems like this can only be fixed by autorefresh!
                self.update_process_table_state();
                return KeyAction::Event(ComponentEvent::ProcessKilled);
            } else {
                return KeyAction::Event(ComponentEvent::ProcessKillFailed);
            }
        }
        KeyAction::Event(ComponentEvent::NoProcessToKill)
    }

    fn enforce_search_by(&mut self, search_by: ProcessRelatedSearch) -> KeyAction {
        let selected_process = self.get_selected_process();
        if selected_process.is_none() {
            return KeyAction::Consumed;
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

        self.search_bar.set_search_text(&search_string);
        self.search_for_processess()
    }

    fn copy_pid_to_clipboard(&mut self) -> KeyAction {
        if let Some(prc) = self.get_selected_process() {
            self.clipboard
                .set_text(format!("{}", prc.pid))
                .expect("Failed to copy pid");
        }
        KeyAction::Consumed
    }
}

impl Component for ProcessesViewComponent {
    fn handle_input(&mut self, key: KeyEvent, action: AppAction) -> KeyAction {
        use KeyCode::*;
        match action {
            AppAction::GoToFirstItem => {
                self.select_first_row();
            }
            AppAction::GoToLastItem => {
                self.select_last_row();
            }
            AppAction::NextItem => {
                self.select_next_row(1);
            }
            AppAction::PreviousItem => {
                self.select_previous_row(1);
            }
            AppAction::JumpTenNextItems => {
                self.select_next_row(10);
            }
            AppAction::JumpTenPreviousItems => {
                self.select_previous_row(10);
            }
            AppAction::KillProcess => {
                return self.kill_selected_process();
            }
            AppAction::RefreshProcessList => {
                return self.search_for_processess();
            }
            AppAction::CopyProcessPid => {
                return self.copy_pid_to_clipboard();
            }
            AppAction::SelectProcessParent => {
                return self.enforce_search_by(ProcessRelatedSearch::Parent);
            }
            AppAction::SelectProcessFamily => {
                return self.enforce_search_by(ProcessRelatedSearch::Family);
            }
            AppAction::SelectProcessSiblings => {
                return self.enforce_search_by(ProcessRelatedSearch::Siblings);
            }
            AppAction::ScrollProcessDetailsUp => {
                self.process_details_component.process_details_up();
            }
            AppAction::ScrollProcessDetailsDown => {
                self.process_details_component.process_details_down();
            }
            //search bar
            AppAction::CursorLeft => {
                self.search_bar.move_cursor(CursorMove::Back);
            }
            AppAction::CursorRight => {
                self.search_bar.move_cursor(CursorMove::Forward);
            }
            AppAction::CursorHome => {
                self.search_bar.move_cursor(CursorMove::Head);
            }
            AppAction::CursorEnd => {
                self.search_bar.move_cursor(CursorMove::End);
            }
            AppAction::DeleteChar => {
                self.search_bar.delete_char();
                return self.search_for_processess();
            }
            AppAction::DeleteNextChar => {
                self.search_bar.delete_next_char();

                return self.search_for_processess();
            }
            AppAction::DeleteWord => {
                self.search_bar.delete_word();
                return self.search_for_processess();
            }
            AppAction::DeleteToStart => {
                self.search_bar.delete_to_start();
                return self.search_for_processess();
            }
            AppAction::Unmapped => {
                if let Char(c) = key.code {
                    self.search_bar.insert_char(c);
                    return self.search_for_processess();
                }
            }
            _ => {
                return KeyAction::Unhandled;
            }
        }
        KeyAction::Consumed
    }

    fn render(&mut self, frame: &mut Frame, layout: &crate::tui::LayoutRects) {
        let selected_index = self.process_table_component.get_selected_process_index();
        let selected_process = self.search_results.nth(selected_index);

        self.search_bar.render(frame, layout);
        self.process_table_component
            .render(frame, layout, &self.search_results);
        self.process_details_component
            .render(frame, layout, selected_process);
    }
}

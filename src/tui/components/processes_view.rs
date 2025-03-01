use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    processes::{IgnoreOptions, Process, ProcessManager, ProcessSearchResults},
    tui::{components::KeyAction, ProcessRelatedSearch},
};

use super::{
    process_details::ProcessDetailsComponent, process_table::ProcessTableComponent, Component,
    ComponentEvent,
};

pub struct ProcessesViewComponent {
    process_manager: ProcessManager,
    ignore_options: IgnoreOptions,
    search_results: ProcessSearchResults,
    process_table_component: ProcessTableComponent,
    process_details_component: ProcessDetailsComponent,
}

impl ProcessesViewComponent {
    pub fn new(use_icons: bool, ignore_options: IgnoreOptions) -> Result<Self> {
        Ok(Self {
            process_manager: ProcessManager::new()?,
            ignore_options,
            search_results: ProcessSearchResults::empty(),
            process_table_component: ProcessTableComponent::new(use_icons),
            process_details_component: ProcessDetailsComponent::new(),
        })
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

    fn search_for_processess(&mut self, search_text: &str) {
        self.process_manager.refresh();
        self.search_results = self
            .process_manager
            .find_processes(search_text, &self.ignore_options);
        self.update_process_table_state();
    }

    fn kill_selected_process(&mut self) -> KeyAction {
        if let Some(prc) = self.get_selected_process() {
            let pid = prc.pid;
            if self.process_manager.kill_process(pid) {
                //TODO: what now???
                // self.search_for_processess(&self.last_search_query);
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
        KeyAction::Event(ComponentEvent::SearchByTextRequested(search_string))
    }
}

impl Component for ProcessesViewComponent {
    fn handle_input(&mut self, key: KeyEvent) -> KeyAction {
        use KeyCode::*;
        match key.code {
            Up if key.modifiers.contains(KeyModifiers::CONTROL) => self.select_first_row(),
            Down if key.modifiers.contains(KeyModifiers::CONTROL) => self.select_last_row(),
            Up | BackTab => self.select_previous_row(1),
            Tab | Down => self.select_next_row(1),
            PageUp => self.select_previous_row(10),
            PageDown => self.select_next_row(10),
            Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.select_next_row(1);
            }
            Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.select_previous_row(1);
            }
            Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return self.kill_selected_process();
            }
            Char('p') if key.modifiers.contains(KeyModifiers::ALT) => {
                return self.enforce_search_by(ProcessRelatedSearch::Parent);
            }
            Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                return self.enforce_search_by(ProcessRelatedSearch::Family);
            }
            Char('s') if key.modifiers.contains(KeyModifiers::ALT) => {
                return self.enforce_search_by(ProcessRelatedSearch::Siblings);
            }

            Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.process_details_component.process_details_down();
            }
            Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.process_details_component.process_details_up();
            }
            _ => return KeyAction::Unhandled,
        };
        KeyAction::Consumed
    }

    fn handle_event(&mut self, event: &super::ComponentEvent) -> Option<super::ComponentEvent> {
        if let ComponentEvent::SearchQueryUpdated(query) = event {
            self.search_for_processess(query);
        };

        None
    }

    fn render(&mut self, frame: &mut ratatui::Frame, layout: &crate::tui::LayoutRects) {
        self.process_table_component
            .render(frame, layout, &self.search_results);
        //TODO: cloning for real or elase fight with rust...
        let selected_process = self.get_selected_process().cloned();
        self.process_details_component
            .render(frame, layout, selected_process.as_ref());
    }
}

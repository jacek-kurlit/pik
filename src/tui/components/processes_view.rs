use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

use anyhow::Result;
use arboard::Clipboard;
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_input::InputRequest;

use crate::config::keymappings::AppAction;
use crate::processes::{
    KilledProcess, OperationResult, Operations, ProcessManager, ProcssAsyncService,
};
use crate::tui::components::search_bar::CursorMove;
use crate::{
    config::ui::UIConfig,
    processes::{IgnoreOptions, Process, ProcessSearchResults},
    tui::{
        ProcessRelatedSearch,
        components::{KeyAction, Notification},
    },
};

use super::{
    Component, ComponentEvent, process_details::ProcessDetailsComponent,
    process_table::ProcessTableComponent, search_bar::SearchBarComponent,
};

pub struct ProcessesViewComponent {
    ops_sender: Sender<Operations>,
    results_receiver: Receiver<OperationResult>,
    search_results: ProcessSearchResults,
    process_table_component: ProcessTableComponent,
    process_details_component: ProcessDetailsComponent,
    search_bar: SearchBarComponent,
}

// NOTE: clipboard access is initialized lazily because some systems do not provide a clipboard
// backend at all. Surface those failures to the user instead of crashing the TUI.
static CLIPBOARD: std::sync::LazyLock<Result<Mutex<Clipboard>, String>> =
    std::sync::LazyLock::new(|| {
        Clipboard::new()
            .map(Mutex::new)
            .map_err(|err| format!("Clipboard is unavailable: {err}"))
    });

impl ProcessesViewComponent {
    pub fn new(
        ui_config: &UIConfig,
        ignore_options: IgnoreOptions,
        initial_query: String,
    ) -> Result<Self> {
        let mut process_service = ProcssAsyncService::new(ProcessManager::new()?, ignore_options);
        let initial_results = process_service.find_processes(&initial_query);
        let (ops_sender, results_receiver) = process_service.run_as_background_process();
        let mut component = Self {
            ops_sender,
            results_receiver,
            search_results: initial_results,
            process_table_component: ProcessTableComponent::new(
                ui_config.icons.get_icons(),
                // cloning for sake of simplicity
                ui_config.process_table.clone(),
            ),
            process_details_component: ProcessDetailsComponent::new(
                ui_config.process_details.clone(),
            ),
            search_bar: SearchBarComponent::new(
                initial_query,
                &ui_config.search_bar,
                ui_config.icons.get_icons().search_prompt.as_str(),
            ),
        };
        component.update_process_table_state();
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

    fn search_for_processess(&mut self) -> Result<(), Notification> {
        let search_text = self.search_bar.get_search_text().to_string();
        match self.ops_sender.send(Operations::Search(search_text)) {
            Ok(_) => Ok(()),
            Err(_) => Err(Notification::error(
                "Failed to send search request to process daemon",
            )),
        }
    }

    fn kill_selected_process(&mut self, graceful: bool) -> KeyAction {
        if let Some(prc) = self.get_selected_process() {
            let pid = prc.pid;
            let name = prc.cmd.clone();
            return match self.ops_sender.send(Operations::KillProcess {
                pid,
                graceful,
                name,
            }) {
                Ok(_) => KeyAction::Consumed,
                Err(_) => KeyAction::Event(ComponentEvent::ShowNotification(Notification::error(
                    "Failed to send kill request to process daemon",
                ))),
            };
        }
        KeyAction::Event(ComponentEvent::ShowNotification(Notification::info(
            "No process selected",
        )))
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
        match self.search_for_processess() {
            Ok(()) => KeyAction::Consumed,
            Err(notification) => KeyAction::Event(ComponentEvent::ShowNotification(notification)),
        }
    }

    fn copy_pid_to_clipboard(&mut self) -> KeyAction {
        if let Some(prc) = self.get_selected_process() {
            let clipboard = match CLIPBOARD.as_ref() {
                Ok(clipboard) => clipboard,
                Err(err) => {
                    return KeyAction::Event(ComponentEvent::ShowNotification(
                        Notification::error(err.clone()),
                    ));
                }
            };
            let mut clipboard = match clipboard.lock() {
                Ok(clipboard) => clipboard,
                Err(_) => {
                    return KeyAction::Event(ComponentEvent::ShowNotification(
                        Notification::error("Clipboard is currently unavailable"),
                    ));
                }
            };
            if let Err(err) = clipboard.set_text(format!("{}", prc.pid)) {
                return KeyAction::Event(ComponentEvent::ShowNotification(Notification::error(
                    format!("Failed to copy PID to clipboard: {err}"),
                )));
            }
        }
        KeyAction::Consumed
    }
}

fn process_result_message(prefix: &str, process: &KilledProcess) -> String {
    let name = if process.name.is_empty() {
        "unknown"
    } else {
        process.name.as_str()
    };

    format!("{prefix} - {name} : PID {}", process.pid)
}

impl Component for ProcessesViewComponent {
    fn update_state(&mut self) -> Option<ComponentEvent> {
        if let Ok(ops_result) = self.results_receiver.try_recv() {
            match ops_result {
                OperationResult::SearchCompleted(results) => {
                    self.search_results = results;
                    self.update_process_table_state();
                }
                OperationResult::ProcessKilled { results, process } => {
                    self.search_results = results;
                    self.update_process_table_state();
                    return Some(ComponentEvent::ShowNotification(Notification::success(
                        process_result_message("Process killed", &process),
                    )));
                }
                OperationResult::ProcessKillFailed(process) => {
                    return Some(ComponentEvent::ShowNotification(Notification::error(
                        process_result_message("Failed to kill process", &process),
                    )));
                }
                OperationResult::Error(err) => {
                    return Some(ComponentEvent::ShowNotification(Notification::error(err)));
                }
            }
        }
        None
    }

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
                return self.kill_selected_process(true);
            }
            AppAction::ForceKillProcess => {
                return self.kill_selected_process(false);
            }
            AppAction::RefreshProcessList => {
                return match self.search_for_processess() {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
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
                self.search_bar
                    .request_input_change(InputRequest::GoToPrevChar);
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
            AppAction::CursorWordLeft => {
                self.search_bar.move_cursor(CursorMove::WordBack);
            }
            AppAction::CursorWordRight => {
                self.search_bar.move_cursor(CursorMove::WordForward);
            }
            AppAction::DeleteChar => {
                self.search_bar.delete_char();
                return match self.search_for_processess() {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteNextChar => {
                self.search_bar.delete_next_char();

                return match self.search_for_processess() {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteWord => {
                self.search_bar.delete_word();
                return match self.search_for_processess() {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteToStart => {
                self.search_bar.delete_to_start();
                return match self.search_for_processess() {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteToEnd => {
                self.search_bar.delete_to_end();
                return match self.search_for_processess() {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteNextWord => {
                self.search_bar.delete_next_word();
                return match self.search_for_processess() {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::Unmapped => {
                if let Char(c) = key.code {
                    self.search_bar.insert_char(c);
                    return match self.search_for_processess() {
                        Ok(()) => KeyAction::Consumed,
                        Err(notification) => {
                            KeyAction::Event(ComponentEvent::ShowNotification(notification))
                        }
                    };
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

impl Drop for ProcessesViewComponent {
    fn drop(&mut self) {
        self.ops_sender.send(Operations::Shutdown).ok();
    }
}

#[cfg(test)]
mod tests {
    use crate::processes::KilledProcess;

    use super::process_result_message;

    #[test]
    fn builds_success_message_with_name_and_pid() {
        let message = process_result_message(
            "Process killed",
            &KilledProcess {
                pid: 4242,
                name: "pik".to_string(),
            },
        );

        assert_eq!(message, "Process killed - pik : PID 4242");
    }

    #[test]
    fn builds_failure_message_with_name_and_pid() {
        let message = process_result_message(
            "Failed to kill process",
            &KilledProcess {
                pid: 4242,
                name: "pik".to_string(),
            },
        );

        assert_eq!(message, "Failed to kill process - pik : PID 4242");
    }

    #[test]
    fn falls_back_to_unknown_name_when_process_name_is_empty() {
        let message = process_result_message(
            "Process killed",
            &KilledProcess {
                pid: 4242,
                name: String::new(),
            },
        );

        assert_eq!(message, "Process killed - unknown : PID 4242");
    }
}

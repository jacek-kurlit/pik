use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

use anyhow::Result;
use arboard::Clipboard;
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_input::InputRequest;

use crate::config::keymappings::AppAction;
use crate::processes::{OperationResult, Operations, ProcessManager, ProcssAsyncService};
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
    show_refresh_result_notification: bool,
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
            show_refresh_result_notification: false,
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

    fn search_for_processess(&mut self, triggered_by_refresh: bool) -> Result<(), Notification> {
        update_refresh_notification_on_search_request(
            &mut self.show_refresh_result_notification,
            triggered_by_refresh,
        );
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
            return match self
                .ops_sender
                .send(Operations::KillProcess { pid, graceful })
            {
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
        match self.search_for_processess(false) {
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

fn update_refresh_notification_on_search_request(
    show_refresh_result_notification: &mut bool,
    triggered_by_refresh: bool,
) {
    if !triggered_by_refresh {
        *show_refresh_result_notification = false;
    }
}

fn notification_for_operation_result(
    result: &OperationResult,
    show_refresh_result_notification: &mut bool,
) -> Option<Notification> {
    match result {
        OperationResult::ProcessKilled(_) => Some(Notification::success("Process killed")),
        OperationResult::ProcessKillFailed => Some(Notification::error(
            "Failed to kill process. Check permissions",
        )),
        OperationResult::Error(error) => {
            *show_refresh_result_notification = false;
            Some(Notification::error(error.clone()))
        }
        OperationResult::SearchCompleted(_) => {
            if *show_refresh_result_notification {
                *show_refresh_result_notification = false;
                Some(Notification::info("Process list refreshed"))
            } else {
                None
            }
        }
    }
}

impl Component for ProcessesViewComponent {
    fn update_state(&mut self) -> Option<ComponentEvent> {
        if let Ok(ops_result) = self.results_receiver.try_recv() {
            match ops_result {
                OperationResult::SearchCompleted(results) => {
                    self.search_results = results;
                    self.update_process_table_state();
                    return notification_for_operation_result(
                        &OperationResult::SearchCompleted(ProcessSearchResults::empty()),
                        &mut self.show_refresh_result_notification,
                    )
                    .map(ComponentEvent::ShowNotification);
                }
                OperationResult::ProcessKilled(results) => {
                    self.search_results = results;
                    self.update_process_table_state();
                    return notification_for_operation_result(
                        &OperationResult::ProcessKilled(ProcessSearchResults::empty()),
                        &mut self.show_refresh_result_notification,
                    )
                    .map(ComponentEvent::ShowNotification);
                }
                OperationResult::ProcessKillFailed => {
                    return notification_for_operation_result(
                        &OperationResult::ProcessKillFailed,
                        &mut self.show_refresh_result_notification,
                    )
                    .map(ComponentEvent::ShowNotification);
                }
                OperationResult::Error(err) => {
                    return notification_for_operation_result(
                        &OperationResult::Error(err),
                        &mut self.show_refresh_result_notification,
                    )
                    .map(ComponentEvent::ShowNotification);
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
                self.show_refresh_result_notification = true;
                return match self.search_for_processess(true) {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        self.show_refresh_result_notification = false;
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
                return match self.search_for_processess(false) {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteNextChar => {
                self.search_bar.delete_next_char();

                return match self.search_for_processess(false) {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteWord => {
                self.search_bar.delete_word();
                return match self.search_for_processess(false) {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteToStart => {
                self.search_bar.delete_to_start();
                return match self.search_for_processess(false) {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteToEnd => {
                self.search_bar.delete_to_end();
                return match self.search_for_processess(false) {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::DeleteNextWord => {
                self.search_bar.delete_next_word();
                return match self.search_for_processess(false) {
                    Ok(()) => KeyAction::Consumed,
                    Err(notification) => {
                        KeyAction::Event(ComponentEvent::ShowNotification(notification))
                    }
                };
            }
            AppAction::Unmapped => {
                if let Char(c) = key.code {
                    self.search_bar.insert_char(c);
                    return match self.search_for_processess(false) {
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
    use crate::processes::{OperationResult, ProcessSearchResults};
    use crate::tui::components::NotificationSeverity;

    use super::{notification_for_operation_result, update_refresh_notification_on_search_request};

    #[test]
    fn maps_process_kill_success_to_success_notification() {
        let mut show_refresh_result_notification = false;
        let notification = notification_for_operation_result(
            &OperationResult::ProcessKilled(ProcessSearchResults::empty()),
            &mut show_refresh_result_notification,
        )
        .unwrap();

        assert_eq!(notification.severity, NotificationSeverity::Success);
        assert_eq!(notification.message, "Process killed");
    }

    #[test]
    fn maps_process_kill_failure_to_error_notification() {
        let mut show_refresh_result_notification = false;
        let notification = notification_for_operation_result(
            &OperationResult::ProcessKillFailed,
            &mut show_refresh_result_notification,
        )
        .unwrap();

        assert_eq!(notification.severity, NotificationSeverity::Error);
        assert_eq!(
            notification.message,
            "Failed to kill process. Check permissions"
        );
    }

    #[test]
    fn maps_daemon_errors_to_error_notification() {
        let mut show_refresh_result_notification = false;
        let notification = notification_for_operation_result(
            &OperationResult::Error("daemon failed".to_string()),
            &mut show_refresh_result_notification,
        )
        .unwrap();

        assert_eq!(notification.severity, NotificationSeverity::Error);
        assert_eq!(notification.message, "daemon failed");
    }

    #[test]
    fn clears_refresh_flag_after_daemon_error() {
        let mut show_refresh_result_notification = true;

        let notification = notification_for_operation_result(
            &OperationResult::Error("daemon failed".to_string()),
            &mut show_refresh_result_notification,
        )
        .unwrap();

        assert_eq!(notification.severity, NotificationSeverity::Error);
        assert!(!show_refresh_result_notification);
    }

    #[test]
    fn does_not_emit_refresh_notification_after_error_then_next_search_completion() {
        let mut show_refresh_result_notification = true;

        notification_for_operation_result(
            &OperationResult::Error("daemon failed".to_string()),
            &mut show_refresh_result_notification,
        );

        let notification = notification_for_operation_result(
            &OperationResult::SearchCompleted(ProcessSearchResults::empty()),
            &mut show_refresh_result_notification,
        );

        assert!(notification.is_none());
        assert!(!show_refresh_result_notification);
    }

    #[test]
    fn non_refresh_search_clears_pending_refresh_notification() {
        let mut show_refresh_result_notification = true;

        update_refresh_notification_on_search_request(&mut show_refresh_result_notification, false);

        assert!(!show_refresh_result_notification);
    }

    #[test]
    fn refresh_search_keeps_pending_refresh_notification() {
        let mut show_refresh_result_notification = true;

        update_refresh_notification_on_search_request(&mut show_refresh_result_notification, true);

        assert!(show_refresh_result_notification);
    }
}

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use anyhow::Result;
use arboard::Clipboard;
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Margin, Rect};
use ratatui::style::palette::tailwind;
use tachyonfx::{CellFilter, EffectManager, Interpolation, fx};
use tui_textarea::CursorMove;

use crate::config::keymappings::AppAction;
use crate::processes::{OperationResult, Operations, ProcessManager, ProcssAsyncService};
use crate::tui;
use crate::tui::fx::UniqueEffectId;
use crate::{
    config::ui::UIConfig,
    processes::{IgnoreOptions, Process, ProcessSearchResults},
    tui::{ProcessRelatedSearch, components::KeyAction},
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
    //TODO: these should not be here
    table_area: Rect,
    effects: EffectManager<UniqueEffectId>,
    anim_started: bool,
    last_frame: Instant,
}

//NOTE: this is wrapped in a Lazy Mutex because arboard's Clipboard may cause issues when you don't
//have any clipboard manager installed, and it needs to be initialized only once.
//FIXME: instead of failing we should send error massage to user
static CLIPBOARD: std::sync::LazyLock<Mutex<Clipboard>> = std::sync::LazyLock::new(|| {
    Mutex::new(Clipboard::new().expect("Failed to create clipboard instance"))
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
            //TODO: remove
            table_area: Rect::ZERO,
            effects: EffectManager::default(),
            anim_started: false,
            last_frame: Instant::now(),
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

    fn search_for_processess(&mut self) -> KeyAction {
        self.play_search_animation();
        let search_text = self.search_bar.get_search_text().to_string();
        match self.ops_sender.send(Operations::Search(search_text)) {
            Ok(_) => KeyAction::Event(ComponentEvent::ProcessListRefreshRequested),
            Err(_) => KeyAction::Event(ComponentEvent::ErrorOccurred(
                "Failed to send search request to process daemon".to_string(),
            )),
        }
    }

    fn play_search_animation(&mut self) {
        // if self.anim_started {
        //     return;
        // }
        // table area
        let area = self.table_area;
        // this should be taken from theme
        // let color = widget.border_color();
        let color = tailwind::BLUE.c400;
        let fx = fx::parallel(&[
            tui::fx::selected_category(color, area),
            fx::fade_from_fg(color, (200, Interpolation::BounceInOut))
                .with_area(area)
                .with_filter(CellFilter::Outer(Margin::new(1, 1))),
        ]);
        self.effects
            .add_unique_effect(UniqueEffectId::SearchStarted, fx);
        self.anim_started = true;
    }

    fn kill_selected_process(&mut self) -> KeyAction {
        if let Some(prc) = self.get_selected_process() {
            let pid = prc.pid;
            return match self.ops_sender.send(Operations::KillProcess(pid)) {
                Ok(_) => KeyAction::Event(ComponentEvent::ProcessKillRequested),
                Err(_) => KeyAction::Event(ComponentEvent::ErrorOccurred(
                    "Failed to send kill request to process daemon".to_string(),
                )),
            };
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
            CLIPBOARD
                .lock()
                .expect("Failed to lock clipboard")
                .set_text(format!("{}", prc.pid))
                .expect("Failed to copy pid");
        }
        KeyAction::Consumed
    }
}

impl Component for ProcessesViewComponent {
    fn update_state(&mut self) -> Option<ComponentEvent> {
        if let Ok(ops_result) = self.results_receiver.try_recv() {
            match ops_result {
                OperationResult::SearchCompleted(results) => {
                    self.search_results = results;
                    self.update_process_table_state();
                    return Some(ComponentEvent::ProcessListRefreshed);
                }
                OperationResult::ProcessKilled(results) => {
                    self.search_results = results;
                    self.update_process_table_state();
                    return Some(ComponentEvent::ProcessKilled);
                }
                OperationResult::ProcessKillFailed => {
                    return Some(ComponentEvent::ProcessKillFailed);
                }
                OperationResult::Error(err) => {
                    eprintln!("Error in process daemon: {err}");
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
        //TODO: remove later
        self.table_area = layout.process_table;
        let elapsed = self.last_frame.elapsed();
        self.last_frame = Instant::now();

        self.search_bar.render(frame, layout);
        self.process_table_component
            .render(frame, layout, &self.search_results);
        self.process_details_component
            .render(frame, layout, selected_process);

        let area = frame.area();
        self.effects
            //TODO: this probably should be area if we want to have one effect manager for entire
            //app
            .process_effects(elapsed.into(), frame.buffer_mut(), area);
    }
}

impl Drop for ProcessesViewComponent {
    fn drop(&mut self) {
        self.ops_sender.send(Operations::Shutdown).ok();
    }
}

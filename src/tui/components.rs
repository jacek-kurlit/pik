use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;

use crate::config::keymappings::AppAction;

use super::LayoutRects;

pub mod debug;
pub mod general_input_handler;
pub mod help_footer;
pub mod help_popup;
pub mod notifications;
pub mod process_details;
pub mod process_table;
pub mod processes_view;
pub mod search_bar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationSeverity {
    Info,
    Success,
    Error,
}

impl NotificationSeverity {
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Info => " Info ",
            Self::Success => " Success ",
            Self::Error => " Error ",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub message: String,
    pub severity: NotificationSeverity,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: NotificationSeverity::Info,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: NotificationSeverity::Success,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: NotificationSeverity::Error,
        }
    }
}

pub trait Component {
    fn handle_input(&mut self, _original: KeyEvent, _app_action: AppAction) -> KeyAction {
        KeyAction::Unhandled
    }

    fn handle_event(&mut self, _: &ComponentEvent) -> Option<ComponentEvent> {
        None
    }

    fn update_state(&mut self) -> Option<ComponentEvent> {
        None
    }

    fn render(&mut self, _frame: &mut Frame, _layout: &LayoutRects) {}
}

pub enum KeyAction {
    // component could not handle key event
    Unhandled,
    // component handled key event but no event is published
    Consumed,
    // component handled key event and event is published
    Event(ComponentEvent),
}

pub enum ComponentEvent {
    QuitRequested,
    ShowNotification(Notification),
}

use std::time::{Duration, Instant};

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph, Wrap},
};

use crate::{config::ui::NotificationsConfig, tui::LayoutRects};

use super::{Component, ComponentEvent, Notification, NotificationSeverity};

struct ActiveNotification {
    notification: Notification,
    shown_at: Instant,
}

pub struct NotificationsComponent {
    active_notification: Option<ActiveNotification>,
    config: NotificationsConfig,
}

impl NotificationsComponent {
    pub fn new(config: &NotificationsConfig) -> Self {
        Self {
            active_notification: None,
            config: config.clone(),
        }
    }

    fn notification_style(&self, severity: &NotificationSeverity) -> Style {
        match severity {
            NotificationSeverity::Info => self.config.theme.info,
            NotificationSeverity::Success => self.config.theme.success,
            NotificationSeverity::Error => self.config.theme.error,
        }
    }

    fn has_expired(&self, shown_at: Instant) -> bool {
        shown_at.elapsed() >= Duration::from_millis(self.config.timeout_ms)
    }
}

impl Component for NotificationsComponent {
    fn handle_event(&mut self, event: &ComponentEvent) -> Option<ComponentEvent> {
        if let ComponentEvent::ShowNotification(notification) = event {
            self.active_notification = Some(ActiveNotification {
                notification: notification.clone(),
                shown_at: Instant::now(),
            });
        }
        None
    }

    fn update_state(&mut self) -> Option<ComponentEvent> {
        if self
            .active_notification
            .as_ref()
            .is_some_and(|notification| self.has_expired(notification.shown_at))
        {
            self.active_notification = None;
        }

        None
    }

    fn render(&mut self, frame: &mut ratatui::Frame, _layout: &LayoutRects) {
        let Some(active_notification) = &self.active_notification else {
            return;
        };

        let style = self.notification_style(&active_notification.notification.severity);
        let area = notification_area(frame.area(), &active_notification.notification.message);
        let block = Block::bordered()
            .title_top(Line::from(Span::styled(
                active_notification.notification.severity.title(),
                style,
            )))
            .border_type(self.config.theme.border._type)
            .border_style(self.config.theme.border.style)
            .padding(Padding::horizontal(1));
        let paragraph = Paragraph::new(active_notification.notification.message.as_str())
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
}

fn notification_area(area: Rect, message: &str) -> Rect {
    let max_width = area.width.saturating_sub(6).clamp(1, 72);
    let min_width = max_width.min(36);
    let width = (message.chars().count() as u16 + 8).clamp(min_width, max_width);
    let inner_width = width.saturating_sub(4).max(1);
    let wrapped_lines = message
        .chars()
        .count()
        .div_ceil(inner_width as usize)
        .max(1);
    let height = (wrapped_lines as u16).min(4) + 2;
    let x = area.right().saturating_sub(width + 3);
    let y = area.y.saturating_add(2);

    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use ratatui::style::{Color, Style};

    use crate::config::ui::{NotificationsConfig, NotificationsTheme};

    use super::*;

    #[test]
    fn replaces_previous_notification_with_latest_one() {
        let mut component = NotificationsComponent::new(&NotificationsConfig::default());
        component.handle_event(&ComponentEvent::ShowNotification(Notification::info(
            "first",
        )));
        component.handle_event(&ComponentEvent::ShowNotification(Notification::error(
            "second",
        )));

        let active = component.active_notification.as_ref().unwrap();
        assert_eq!(active.notification.message, "second");
        assert_eq!(active.notification.severity, NotificationSeverity::Error);
    }

    #[test]
    fn clears_notification_after_timeout() {
        let mut component = NotificationsComponent::new(&NotificationsConfig {
            timeout_ms: 1,
            ..NotificationsConfig::default()
        });
        component.active_notification = Some(ActiveNotification {
            notification: Notification::info("expired"),
            shown_at: Instant::now() - Duration::from_millis(5),
        });

        component.update_state();

        assert!(component.active_notification.is_none());
    }

    #[test]
    fn uses_severity_specific_style() {
        let theme = NotificationsTheme {
            info: Style::new().fg(Color::Blue),
            success: Style::new().fg(Color::Green),
            error: Style::new().fg(Color::Red),
            ..NotificationsTheme::default()
        };
        let component = NotificationsComponent::new(&NotificationsConfig {
            theme: theme.clone(),
            ..NotificationsConfig::default()
        });

        assert_eq!(
            component.notification_style(&NotificationSeverity::Info),
            theme.info
        );
        assert_eq!(
            component.notification_style(&NotificationSeverity::Success),
            theme.success
        );
        assert_eq!(
            component.notification_style(&NotificationSeverity::Error),
            theme.error
        );
    }
}

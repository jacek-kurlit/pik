use std::time::Instant;

use ratatui::{Frame, crossterm::event::KeyEvent, widgets::Paragraph};

use crate::{config::keymappings::AppAction, tui::LayoutRects};

use super::{Component, KeyAction};

pub struct FpsCounter {
    show: bool,
    renders_count: usize,
    last_measurement: Instant,
    fps_text: Paragraph<'static>,
}

#[allow(clippy::new_without_default)]
impl FpsCounter {
    pub fn new() -> Self {
        Self {
            show: false,
            renders_count: 0,
            fps_text: Paragraph::new("FPS: 0.00").centered(),
            last_measurement: Instant::now(),
        }
    }

    pub fn toggle(&mut self) {
        self.show = !self.show;
        self.last_measurement = Instant::now();
        self.renders_count = 0;
    }
}

const ONE_SECOND: u128 = 1000;

impl Component for FpsCounter {
    fn handle_input(&mut self, _: KeyEvent, action: AppAction) -> KeyAction {
        if matches!(action, AppAction::ToggleFps) {
            self.toggle();
            return KeyAction::Consumed;
        }
        KeyAction::Unhandled
    }

    fn render(&mut self, frame: &mut Frame, layout: &LayoutRects) {
        if !self.show {
            return;
        }
        self.renders_count += 1;
        let now = Instant::now();
        let millis_elapsed = self.last_measurement.elapsed().as_millis();
        // once second passed since last measurements
        if millis_elapsed > ONE_SECOND {
            let fps = self.renders_count as f32 * 1000.0 / millis_elapsed as f32;
            self.fps_text = Paragraph::new(format!("FPS: {fps:.2}")).centered();
            self.last_measurement = now;
            self.renders_count = 0;
        }

        frame.render_widget(&self.fps_text, layout.fps_counter);
    }
}

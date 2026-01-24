use std::{
    collections::VecDeque,
    io::{self, stdout},
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use components::{
    Component, ComponentEvent, KeyAction, debug::DebugComponent,
    general_input_handler::GeneralInputHandlerComponent, help_footer::HelpFooterComponent,
    help_popup::HelpPopupComponent, processes_view::ProcessesViewComponent,
};
use ratatui::crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::EnterAlternateScreen,
};
use ratatui::{TerminalOptions, prelude::*};

pub mod components;
mod highlight;

use crate::{config::keymappings::KeyMappings, settings::AppSettings};

struct App {
    components: Vec<Box<dyn Component>>,
    component_events: VecDeque<ComponentEvent>,
    key_mappings: KeyMappings,
}

// NOTE: Simple FPS limiter based on input read delay (it's not stable, typing really fast will increase FPS for short time)
const FPS: u64 = 60;
//setting this to 0 will make the app run as fast as possible causing high CPU usage for rendering
const KEY_READ_DELAY: u64 = 1000 / FPS;

impl App {
    fn new(app_settings: AppSettings) -> Result<App> {
        let component_events = VecDeque::new();

        Ok(App {
            //Order matters!
            //Input handling is done in this order
            //Rendering is done in reverse
            //It allows for popups to be rendered on top but they handle input first
            components: vec![
                Box::new(HelpPopupComponent::new(
                    &app_settings.ui_config,
                    &app_settings.key_mappings,
                )),
                Box::new(GeneralInputHandlerComponent),
                Box::new(HelpFooterComponent::new(&app_settings.key_mappings)),
                Box::new(DebugComponent::new()),
                Box::new(ProcessesViewComponent::new(
                    &app_settings.ui_config,
                    app_settings.filter_opions,
                    app_settings.query,
                )?),
            ],
            component_events,
            key_mappings: app_settings.key_mappings,
        })
    }

    fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            self.handle_input()?;
            self.update_state();
            if self.handle_events()? {
                return Ok(());
            }

            self.render(terminal)?;
        }
    }

    fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(KEY_READ_DELAY))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            let action = self.key_mappings.resolve(key);
            for component in self.components.iter_mut() {
                let action = component.handle_input(key, action);
                match action {
                    KeyAction::Unhandled => continue,
                    KeyAction::Consumed => {
                        break;
                    }
                    KeyAction::Event(act) => {
                        self.component_events.push_back(act);
                        break;
                    }
                }
            }
        };
        Ok(())
    }

    fn update_state(&mut self) {
        for component in self.components.iter_mut() {
            if let Some(event) = component.update_state() {
                self.component_events.push_back(event);
            }
        }
    }

    fn handle_events(&mut self) -> Result<bool> {
        while let Some(event) = self.component_events.pop_front() {
            if let ComponentEvent::QuitRequested = event {
                return Ok(true);
            }
            for component in self.components.iter_mut() {
                let new_event = component.handle_event(&event);
                if let Some(new_event) = new_event {
                    self.component_events.push_back(new_event);
                }
            }
        }
        Ok(false)
    }

    fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal
            .draw(|frame| {
                let layout = LayoutRects::new(frame);
                for component in self.components.iter_mut().rev() {
                    component.render(frame, &layout);
                }
            })
            .map_err(|e| anyhow!("Failed to render on terminal {e}"))?;
        Ok(())
    }
}

pub enum ProcessRelatedSearch {
    Family,   // process + process children
    Siblings, // process parent + all his children
    Parent,   // only parent process
}

pub fn start_app(app_settings: AppSettings) -> Result<()> {
    // setup terminal
    let viewport = app_settings.viewport.clone();
    if matches!(viewport, ratatui::Viewport::Fullscreen) {
        execute!(stdout(), EnterAlternateScreen)?;
    }

    set_cursor_color(app_settings.ui_config.search_bar.cursor_style.bg)?;

    let mut terminal = ratatui::init_with_options(TerminalOptions { viewport });

    // create app and run it
    let app = App::new(app_settings)?;
    let res = app.run(&mut terminal);

    // restore terminal
    terminal.clear()?;
    ratatui::restore();
    reset_cursor_color()?;

    //FIXME: add error handling, for example some error page should be shown
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

use std::io::Write;

fn set_cursor_color(color: Option<Color>) -> Result<()> {
    if let Some(color) = color {
        // OSC 12 ; color BEL
        write!(io::stdout(), "\x1b]12;{}\x07", color).context("failed to set cursor color")?;
        io::stdout().flush().context("failed to set cursor color")?;
    }
    Ok(())
}

fn reset_cursor_color() -> Result<()> {
    // OSC 112 resets cursor color
    write!(io::stdout(), "\x1b]112\x07").context("failed to reset cursor color")?;
    io::stdout().flush().context("failed to reset cursor color")
}

pub struct LayoutRects {
    pub top_bar: Rect,
    pub process_table: Rect,
    pub process_details: Rect,
    pub debug: Rect,
    pub help_text: Rect,
}

impl LayoutRects {
    pub fn new(frame: &Frame) -> Self {
        let rects = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Max(7),
            Constraint::Length(1),
        ])
        .split(frame.area());
        let footer = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rects[3]);
        Self {
            top_bar: rects[0],
            process_table: rects[1],
            process_details: rects[2],
            debug: footer[0],
            help_text: footer[1],
        }
    }
}

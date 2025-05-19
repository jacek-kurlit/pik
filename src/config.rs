use anyhow::{Context, Ok, Result};

pub mod keymappings;
pub mod ui;

pub fn load_app_config() -> Result<AppConfig> {
    let config_path = directories::ProjectDirs::from("", "", "pik")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .filter(|path| path.exists());

    match config_path {
        Some(path) => load_config_from_file(&path),
        None => Ok(AppConfig::default()),
    }
}

fn load_config_from_file(path: &std::path::PathBuf) -> Result<AppConfig> {
    let toml = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to load config from file: {:?}", path))?;
    parse_config(&toml)
}

fn parse_config(toml: &str) -> Result<AppConfig> {
    let mut config: AppConfig = toml::from_str(toml)
        .with_context(|| format!("Failed to deserialize config from: {:?}", toml))?;

    config.key_mappings =
        KeyMappings::preconfigured_mappings().override_with(config.key_mappings)?;
    Ok(config)
}

use keymappings::KeyMappings;
use regex::Regex;
use serde::Deserialize;
use ui::UIConfig;

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub screen_size: ScreenSize,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub key_mappings: KeyMappings,
    #[serde(default)]
    pub ui: UIConfig,
}

#[derive(Debug, Deserialize)]
pub struct IgnoreConfig {
    #[serde(with = "serde_regex", default)]
    pub paths: Vec<Regex>,
    #[serde(default = "set_true")]
    pub other_users: bool,
    #[serde(default = "set_true")]
    pub threads: bool,
}

const fn set_true() -> bool {
    true
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            paths: vec![],
            other_users: set_true(),
            threads: set_true(),
        }
    }
}

impl PartialEq for IgnoreConfig {
    fn eq(&self, other: &Self) -> bool {
        let mut eq = self.threads == other.threads
            && self.other_users == other.other_users
            && self.paths.len() == other.paths.len();
        if eq {
            eq = self.paths.iter().map(|r| r.as_str()).collect::<Vec<&str>>()
                == other
                    .paths
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<&str>>()
        }
        eq
    }
}

impl Eq for IgnoreConfig {}

#[derive(Debug, Eq, PartialEq, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ScreenSize {
    Fullscreen,
    Height(u16),
}

pub const DEFAULT_SCREEN_SIZE: u16 = 25;

impl Default for ScreenSize {
    fn default() -> Self {
        ScreenSize::Height(DEFAULT_SCREEN_SIZE)
    }
}

#[cfg(test)]
mod tests {

    use ratatui::{
        crossterm::event::{KeyCode, KeyModifiers},
        layout::{Alignment, Margin},
        style::{
            Color, Modifier, Style, Stylize,
            palette::tailwind::{self, SLATE},
        },
        widgets::{BorderType, block::Position},
    };
    use ui::{
        BorderTheme, CellTheme, ProcessDetailsTheme, RowTheme, ScrollbarTheme, SearchBarTheme,
        TableTheme, TitleTheme,
    };

    use crate::config::{
        keymappings::{AppAction, KeyBinding},
        ui::PopupsTheme,
    };

    use super::*;

    #[test]
    fn should_deserialize_empty_configuration() {
        let default_settings = parse_config("").expect("Parsing empty string should work");
        assert_eq!(
            default_settings,
            AppConfig {
                screen_size: ScreenSize::Height(DEFAULT_SCREEN_SIZE),
                ignore: IgnoreConfig {
                    paths: vec![],
                    other_users: true,
                    threads: true
                },
                key_mappings: KeyMappings::preconfigured_mappings(),
                ui: UIConfig {
                    icons: ui::IconConfig::Ascii,
                    process_table: TableTheme {
                        title: TitleTheme {
                            alignment: Alignment::Left,
                            position: Position::Top
                        },
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::BLUE.c400),
                            _type: BorderType::Rounded
                        },
                        row: RowTheme {
                            even: Style::new()
                                .bg(tailwind::SLATE.c950)
                                .fg(tailwind::SLATE.c200),
                            odd: Style::new()
                                .bg(tailwind::SLATE.c900)
                                .fg(tailwind::SLATE.c200),
                            selected: Style::new()
                                .fg(tailwind::BLUE.c400)
                                .add_modifier(Modifier::REVERSED),
                            selected_symbol: " ".to_string(),
                        },
                        cell: CellTheme {
                            normal: Style::default(),
                            highlighted: Style::new().bg(Color::Yellow).italic(),
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::default(),
                            thumb_symbol: None,
                            track_symbol: Some("│".to_string()),
                            begin_symbol: Some("↑".to_string()),
                            end_symbol: Some("↓".to_string()),
                            margin: Margin {
                                vertical: 1,
                                horizontal: 0,
                            },
                        }
                    },
                    process_details: ProcessDetailsTheme {
                        title: TitleTheme {
                            alignment: Alignment::Left,
                            position: Position::Top
                        },
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::BLUE.c400),
                            _type: BorderType::Rounded
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::default(),
                            thumb_symbol: None,
                            track_symbol: Some("│".to_string()),
                            begin_symbol: Some("↑".to_string()),
                            end_symbol: Some("↓".to_string()),
                            margin: Margin {
                                vertical: 1,
                                horizontal: 0
                            }
                        }
                    },
                    search_bar: SearchBarTheme {
                        style: Style::default(),
                        cursor_style: Style::default().add_modifier(Modifier::REVERSED)
                    },
                    popups: PopupsTheme {
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::GREEN.c400),
                            _type: BorderType::Rounded
                        },
                        selected_row: Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD),
                        primary: Style::new().fg(tailwind::BLUE.c400),
                        secondary: Style::default(),
                    }
                }
            }
        );
    }

    #[test]
    fn should_allow_to_override_defaults() {
        let overrided_settings: AppConfig = parse_config(
            r##"
            screen_size = "fullscreen"

            [ignore]
            paths=["/usr/*"]
            other_users = false
            threads = false

            [key_mappings]
            quit = ["ctrl+c", "alt+c"]
            close = ["enter"]

            [ui]
            use_icons = true
            icons = "nerd_font_v3"

            [ui.process_table.title]
            alignment = "right"
            position = "bottom"

            [ui.process_table.border]
            type = "plain"
            style = {fg = "#6366f1", add_modifier = "BOLD | ITALIC"}

            [ui.process_table.row]
            selected_symbol = ">"
            even = {fg = "#fafaf9", bg = "#57534e", add_modifier = "BOLD"}
            odd = {fg = "#ecfdf5", bg = "#059669", add_modifier = "ITALIC"}
            selected = {fg = "#f87171"}

            [ui.process_table.cell]
            normal = {fg = "#a5f3fc", bg = "#0891b2", add_modifier = "CROSSED_OUT"}
            highlighted = {fg = "#fff7ed", bg = "#fb923c", add_modifier = "UNDERLINED"}

            [ui.process_table.scrollbar]
            style = {fg = "#f472b6", bg = "#4c1d95", add_modifier = "BOLD"}
            thumb_symbol = "x"
            track_symbol = "y"
            begin_symbol = "z"
            end_symbol = "q"
            margin = {horizontal = 10, vertical = 20}

            [ui.process_details.title]
            alignment = "center"
            position = "bottom"

            [ui.process_details.border]
            type = "double"
            style = {fg = "#6366f1", add_modifier = "UNDERLINED | ITALIC"}

            [ui.process_details.scrollbar]
            style = {fg = "#f472b6", bg = "#4c1d95", add_modifier = "BOLD"}
            thumb_symbol = "T"
            track_symbol = "="
            begin_symbol = "^"
            end_symbol = "v"
            margin = {horizontal = 2, vertical = 3}

            [ui.search_bar]
            style = {fg = "#6366f1", add_modifier = "UNDERLINED | ITALIC"}
            cursor_style = {fg = "#a5f3fc", bg = "#0891b2", add_modifier = "CROSSED_OUT"}

            [ui.popups]
            border = {type = "plain", style = {fg = "#6366f1", add_modifier = "BOLD | ITALIC"}}
            selected_row = {fg = "#57534e", bg = "#fafaf9", add_modifier = "ITALIC"}
            primary = {fg = "#f472b6", bg = "#4c1d95", add_modifier = "BOLD"}
            secondary = {fg = "#a5f3fc", bg = "#0891b2", add_modifier = "CROSSED_OUT"}
            "##,
        )
        .expect("This should be parseable");
        let mut overrides = KeyMappings::new();
        overrides.insert(
            AppAction::Quit,
            vec![
                KeyBinding::char_with_mod('c', KeyModifiers::CONTROL),
                KeyBinding::char_with_mod('c', KeyModifiers::ALT),
            ],
        );
        overrides.insert(AppAction::Close, vec![KeyBinding::key(KeyCode::Enter)]);
        let key_mappings = KeyMappings::preconfigured_mappings()
            .override_with(overrides)
            .expect("This should be valid");
        assert_eq!(
            overrided_settings,
            AppConfig {
                screen_size: ScreenSize::Fullscreen,
                ignore: IgnoreConfig {
                    paths: vec![Regex::new("/usr/*").unwrap()],
                    other_users: false,
                    threads: false
                },
                key_mappings,
                ui: UIConfig {
                    icons: ui::IconConfig::NerdFontV3,
                    process_table: TableTheme {
                        title: TitleTheme {
                            alignment: Alignment::Right,
                            position: Position::Bottom
                        },
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::INDIGO.c500).bold().italic(),
                            _type: BorderType::Plain
                        },
                        row: RowTheme {
                            even: Style::new()
                                .fg(tailwind::STONE.c50)
                                .bg(tailwind::STONE.c600)
                                .bold(),
                            odd: Style::new()
                                .fg(tailwind::EMERALD.c50)
                                .bg(tailwind::EMERALD.c600)
                                .italic(),
                            selected: Style::new().fg(tailwind::RED.c400),
                            selected_symbol: ">".to_string(),
                        },
                        cell: CellTheme {
                            normal: Style::new()
                                .fg(tailwind::CYAN.c200)
                                .bg(tailwind::CYAN.c600)
                                .crossed_out(),
                            highlighted: Style::new()
                                .fg(tailwind::ORANGE.c50)
                                .bg(tailwind::ORANGE.c400)
                                .underlined(),
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::new()
                                .fg(tailwind::PINK.c400)
                                .bg(tailwind::VIOLET.c900)
                                .bold(),
                            thumb_symbol: Some("x".to_string()),
                            track_symbol: Some("y".to_string()),
                            begin_symbol: Some("z".to_string()),
                            end_symbol: Some("q".to_string()),
                            margin: Margin {
                                vertical: 20,
                                horizontal: 10
                            }
                        }
                    },
                    process_details: ProcessDetailsTheme {
                        title: TitleTheme {
                            alignment: Alignment::Center,
                            position: Position::Bottom
                        },
                        border: BorderTheme {
                            style: Style::default()
                                .fg(tailwind::INDIGO.c500)
                                .italic()
                                .underlined(),
                            _type: BorderType::Double
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::default()
                                .fg(tailwind::PINK.c400)
                                .bg(tailwind::VIOLET.c900)
                                .bold(),
                            thumb_symbol: Some("T".to_string()),
                            track_symbol: Some("=".to_string()),
                            begin_symbol: Some("^".to_string()),
                            end_symbol: Some("v".to_string()),
                            margin: Margin {
                                horizontal: 2,
                                vertical: 3
                            }
                        }
                    },
                    search_bar: SearchBarTheme {
                        style: Style::default()
                            .fg(tailwind::INDIGO.c500)
                            .italic()
                            .underlined(),
                        cursor_style: Style::default()
                            .fg(tailwind::CYAN.c200)
                            .bg(tailwind::CYAN.c600)
                            .crossed_out(),
                    },
                    popups: PopupsTheme {
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::INDIGO.c500).bold().italic(),
                            _type: BorderType::Plain
                        },
                        selected_row: Style::new()
                            .fg(tailwind::STONE.c600)
                            .bg(tailwind::STONE.c50)
                            .italic(),
                        primary: Style::default()
                            .fg(tailwind::PINK.c400)
                            .bg(tailwind::VIOLET.c900)
                            .bold(),
                        secondary: Style::new()
                            .fg(tailwind::CYAN.c200)
                            .bg(tailwind::CYAN.c600)
                            .crossed_out(),
                    }
                }
            }
        );
    }
}

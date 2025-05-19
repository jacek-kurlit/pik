use std::sync::OnceLock;

use ratatui::{
    layout::{Alignment, Margin},
    style::{
        Color, Modifier, Style, Stylize,
        palette::tailwind::{self, SLATE},
    },
    widgets::{BorderType, block::Position},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct UIConfig {
    #[serde(default)]
    pub icons: IconConfig,
    #[serde(default)]
    pub process_table: TableTheme,
    #[serde(default)]
    pub process_details: ProcessDetailsTheme,
    #[serde(default)]
    pub search_bar: SearchBarTheme,
    #[serde(default)]
    pub popups: PopupsTheme,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct IconsStruct {
    pub user: String,
    pub pid: String,
    pub parent: String,
    pub time: String,
    pub cmd: String,
    pub path: String,
    pub args: String,
    pub ports: String,
    pub search_prompt: String,
}

impl IconsStruct {
    pub fn ascii() -> Self {
        Self {
            search_prompt: ">".to_string(),
            ..Default::default()
        }
    }

    pub fn nerd_font_v3() -> Self {
        Self {
            user: "󰋦".to_string(),
            pid: "".to_string(),
            parent: "󱖁".to_string(),
            time: "".to_string(),
            cmd: "󱃸".to_string(),
            path: "".to_string(),
            args: "󱃼".to_string(),
            ports: "".to_string(),
            search_prompt: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum IconConfig {
    #[default]
    Ascii,
    NerdFontV3,
    Custom(IconsStruct),
}

static ASCII_CONFIG: OnceLock<IconsStruct> = OnceLock::new();
static NERD_FONT_V3_CONFIG: OnceLock<IconsStruct> = OnceLock::new();

impl IconConfig {
    pub fn get_icons(&self) -> &IconsStruct {
        match self {
            IconConfig::Ascii => ASCII_CONFIG.get_or_init(IconsStruct::ascii),
            IconConfig::NerdFontV3 => NERD_FONT_V3_CONFIG.get_or_init(IconsStruct::nerd_font_v3),
            IconConfig::Custom(icons) => icons,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct TableTheme {
    #[serde(default)]
    pub title: TitleTheme,
    #[serde(default)]
    pub border: BorderTheme,
    #[serde(default)]
    pub row: RowTheme,
    #[serde(default)]
    pub cell: CellTheme,
    #[serde(default)]
    pub scrollbar: ScrollbarTheme,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct BorderTheme {
    #[serde(default, with = "StyleDef")]
    pub style: Style,
    #[serde(default, with = "BorderTypeDef", rename = "type")]
    pub _type: BorderType,
}

impl Default for BorderTheme {
    fn default() -> Self {
        Self {
            style: Style::default().fg(tailwind::BLUE.c400),
            _type: BorderType::Rounded,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct TitleTheme {
    #[serde(default, with = "AlignmentDef")]
    pub alignment: Alignment,
    #[serde(default, with = "PositionDef")]
    pub position: Position,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct RowTheme {
    #[serde(default, with = "StyleDef")]
    pub even: Style,
    #[serde(default, with = "StyleDef")]
    pub odd: Style,
    #[serde(default, with = "StyleDef")]
    pub selected: Style,
    #[serde(default)]
    pub selected_symbol: String,
}

impl Default for RowTheme {
    fn default() -> Self {
        Self {
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CellTheme {
    #[serde(default, with = "StyleDef")]
    pub normal: Style,
    #[serde(default, with = "StyleDef")]
    pub highlighted: Style,
}

impl Default for CellTheme {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            highlighted: Style::new().bg(Color::Yellow).italic(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(remote = "Style")]
pub struct StyleDef {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub underline_color: Option<Color>,
    #[serde(default)]
    pub add_modifier: Modifier,
    #[serde(default)]
    pub sub_modifier: Modifier,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(remote = "BorderType")]
pub enum BorderTypeDef {
    #[default]
    #[serde(alias = "plain")]
    Plain,
    #[serde(alias = "rounded")]
    Rounded,
    #[serde(alias = "double")]
    Double,
    #[serde(alias = "thick")]
    Thick,
    #[serde(alias = "quadrant_inside")]
    QuadrantInside,
    #[serde(alias = "quadrant_outside")]
    QuadrantOutside,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(remote = "Alignment")]
pub enum AlignmentDef {
    #[default]
    #[serde(alias = "left")]
    Left,
    #[serde(alias = "center")]
    Center,
    #[serde(alias = "right")]
    Right,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(remote = "Position")]
pub enum PositionDef {
    #[default]
    #[serde(alias = "top")]
    Top,
    #[serde(alias = "bottom")]
    Bottom,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ScrollbarTheme {
    #[serde(default, with = "StyleDef")]
    pub style: Style,
    pub thumb_symbol: Option<String>,
    pub track_symbol: Option<String>,
    pub begin_symbol: Option<String>,
    pub end_symbol: Option<String>,
    #[serde(default)]
    pub margin: Margin,
}

impl Default for ScrollbarTheme {
    fn default() -> Self {
        Self {
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
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct ProcessDetailsTheme {
    #[serde(default)]
    pub title: TitleTheme,
    #[serde(default)]
    pub border: BorderTheme,
    #[serde(default)]
    pub scrollbar: ScrollbarTheme,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SearchBarTheme {
    #[serde(default, with = "StyleDef")]
    pub style: Style,
    #[serde(default, with = "StyleDef")]
    pub cursor_style: Style,
}

impl Default for SearchBarTheme {
    fn default() -> Self {
        Self {
            style: Style::default(),
            cursor_style: Style::default().add_modifier(Modifier::REVERSED),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PopupsTheme {
    #[serde(default)]
    pub border: BorderTheme,
    #[serde(default, with = "StyleDef")]
    pub selected_row: Style,
    #[serde(default, with = "StyleDef")]
    pub primary: Style,
    #[serde(default, with = "StyleDef")]
    pub secondary: Style,
}

impl Default for PopupsTheme {
    fn default() -> Self {
        Self {
            border: BorderTheme {
                style: Style::new().fg(tailwind::GREEN.c400),
                ..Default::default()
            },
            selected_row: Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD),
            primary: Style::new().fg(tailwind::BLUE.c400),
            secondary: Style::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_accept_snake_case() {
        let toml = r#"
            [title]
            alignment = "right"
            position = "bottom"
            [border]
            type = "quadrant_inside"
            style = {}
        "#;

        let config: TableTheme = toml::from_str(toml).unwrap();
        assert_eq!(config.title.alignment, Alignment::Right);
        assert_eq!(config.title.position, Position::Bottom);
        assert_eq!(config.border._type, BorderType::QuadrantInside);
    }

    #[test]
    fn enums_accept_pascal_case() {
        let toml = r#"
            [title]
            alignment = "Right"
            position = "Bottom"
            [border]
            type = "QuadrantInside"
            style = {}
        "#;

        let config: TableTheme = toml::from_str(toml).unwrap();
        assert_eq!(config.title.alignment, Alignment::Right);
        assert_eq!(config.title.position, Position::Bottom);
        assert_eq!(config.border._type, BorderType::QuadrantInside);
    }

    #[test]
    fn test_icons_struct_ascii() {
        let icons = IconsStruct::ascii();

        assert_eq!(icons.search_prompt, ">".to_string());
        assert_eq!(icons.user, "".to_string());
        assert_eq!(icons.parent, "".to_string());
        assert_eq!(icons.time, "".to_string());
        assert_eq!(icons.cmd, "".to_string());
        assert_eq!(icons.path, "".to_string());
        assert_eq!(icons.args, "".to_string());
        assert_eq!(icons.ports, "".to_string());
    }

    #[test]
    fn test_icons_struct_nerd_font_v3() {
        let icons = IconsStruct::nerd_font_v3();

        assert_eq!(icons.user, "󰋦".to_string());
        assert_eq!(icons.pid, "".to_string());
        assert_eq!(icons.parent, "󱖁".to_string());
        assert_eq!(icons.time, "".to_string());
        assert_eq!(icons.cmd, "󱃸".to_string());
        assert_eq!(icons.path, "".to_string());
        assert_eq!(icons.args, "󱃼".to_string());
        assert_eq!(icons.ports, "".to_string());
        assert_eq!(icons.search_prompt, "".to_string());
    }

    #[test]
    fn test_icon_config_get_icons() {
        // Test Ascii variant
        let ascii_config = IconConfig::Ascii;
        let ascii_icons = ascii_config.get_icons();
        assert_eq!(ascii_icons, &IconsStruct::ascii());

        // Test NerdFontV3 variant
        let nerd_font_config = IconConfig::NerdFontV3;
        let nerd_font_icons = nerd_font_config.get_icons();
        assert_eq!(nerd_font_icons, &IconsStruct::nerd_font_v3());

        // Test Custom variant
        let custom_icons = IconsStruct {
            user: "custom".to_string(),
            ..Default::default()
        };
        let custom_config = IconConfig::Custom(custom_icons.clone());
        let returned_icons = custom_config.get_icons();
        assert_eq!(returned_icons, &custom_icons);
    }

    #[test]
    fn test_icon_config_default() {
        // Test that the default for IconConfig is Ascii
        let default_config = IconConfig::default();
        assert_eq!(default_config, IconConfig::Ascii);
    }
}

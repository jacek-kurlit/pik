use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style, palette::tailwind},
    widgets::{BorderType, block::Position},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct UIConfig {
    #[serde(default)]
    pub use_icons: bool,
    #[serde(default)]
    pub process_table: TableTheme,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct TableTheme {
    #[serde(default)]
    pub title: TitleTheme,
    #[serde(default)]
    pub border: BorderTheme,
    #[serde(default)]
    pub row: RowTheme,
    #[serde(default)]
    pub cell: CellTheme,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BorderTheme {
    #[serde(with = "StyleDef")]
    pub style: Style,
    #[serde(with = "BorderTypeDef", rename = "type")]
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

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct TitleTheme {
    #[serde(with = "AlignmentDef")]
    pub alignment: Alignment,
    #[serde(with = "PositionDef")]
    pub position: Position,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SelectedRowTheme {
    #[serde(with = "StyleDef")]
    pub style: Style,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RowTheme {
    #[serde(with = "StyleDef")]
    pub even: Style,
    #[serde(with = "StyleDef")]
    pub odd: Style,
    #[serde(with = "StyleDef")]
    pub selected: Style,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CellTheme {
    #[serde(with = "StyleDef")]
    pub normal: Style,
    #[serde(with = "StyleDef")]
    pub highlighted: Style,
}

impl Default for CellTheme {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            highlighted: Style::new().bg(Color::Yellow).fg(Color::Black),
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
    Plain,
    Rounded,
    Double,
    Thick,
    QuadrantInside,
    QuadrantOutside,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(remote = "Alignment")]
pub enum AlignmentDef {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(remote = "Position")]
pub enum PositionDef {
    #[default]
    Top,
    Bottom,
}

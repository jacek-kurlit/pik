use pik::config::{default_config, parse_config};
use ratatui::{
    layout::Margin,
    style::{Color, Modifier, Style, palette::tailwind},
};

#[test]
fn empty_config_uses_embedded_defaults() {
    assert_eq!(parse_config("").unwrap(), default_config().unwrap());
}

#[test]
fn partial_scrollbar_override_preserves_other_defaults() {
    let config = parse_config(
        r#"
        [ui.process_details.scrollbar]
        thumb_symbol = "T"
        "#,
    )
    .unwrap();

    let scrollbar = &config.ui.process_details.scrollbar;

    assert_eq!(scrollbar.thumb_symbol.as_deref(), Some("T"));
    assert_eq!(scrollbar.track_symbol.as_deref(), Some("│"));
    assert_eq!(scrollbar.begin_symbol.as_deref(), Some("↑"));
    assert_eq!(scrollbar.end_symbol.as_deref(), Some("↓"));
    assert_eq!(
        scrollbar.margin,
        Margin {
            horizontal: 0,
            vertical: 1,
        }
    );
}

#[test]
fn inline_tables_are_merged_recursively() {
    let config = parse_config(
        r##"
        [ui.process_details.scrollbar]
        margin = { horizontal = 2 }

        [ui.popups.border]
        style = { bg = "#4c1d95" }

        [ui.notifications.theme]
        success = { bg = "#14532d" }
        "##,
    )
    .unwrap();

    assert_eq!(
        config.ui.process_details.scrollbar.margin,
        Margin {
            horizontal: 2,
            vertical: 1,
        }
    );
    assert_eq!(
        config.ui.popups.border.style,
        Style::new()
            .fg(tailwind::GREEN.c400)
            .bg(tailwind::VIOLET.c900)
    );
    assert_eq!(
        config.ui.notifications.theme.success,
        Style::new()
            .fg(tailwind::GREEN.c400)
            .bg(Color::Rgb(20, 83, 45))
    );
}

#[test]
fn key_mapping_arrays_replace_single_actions_without_affecting_others() {
    let config = parse_config(
        r#"
        [key_mappings]
        quit = ["ctrl+c", "alt+c"]
        "#,
    )
    .unwrap();

    assert_eq!(
        config
            .key_mappings
            .get_joined(pik::config::keymappings::AppAction::Quit, "/"),
        "ctrl+c/alt+c"
    );
    assert_eq!(
        config
            .key_mappings
            .get_joined(pik::config::keymappings::AppAction::Close, "/"),
        "esc"
    );
}

#[test]
fn scalar_values_replace_default_tables_when_shape_changes() {
    let config = parse_config("screen_size = \"fullscreen\"").unwrap();

    assert_eq!(config.screen_size, pik::config::ScreenSize::Fullscreen);
}

#[test]
fn partial_style_override_keeps_default_modifiers() {
    let config = parse_config(
        r##"
        [ui.process_table.row]
        selected = { fg = "#f87171" }
        "##,
    )
    .unwrap();

    assert_eq!(
        config.ui.process_table.row.selected,
        Style::new()
            .fg(tailwind::RED.c400)
            .add_modifier(Modifier::REVERSED)
    );
}

#[test]
fn partial_style_override_inherits_default_reversed_modifier() {
    let config = parse_config(
        r##"
        [ui.process_table.row]
        selected = { fg = "#FFFFFF" }
        "##,
    )
    .unwrap();

    assert_eq!(
        config.ui.process_table.row.selected,
        Style::new()
            .fg(tailwind::WHITE)
            .add_modifier(Modifier::REVERSED)
    );
}

#[test]
fn full_style_block_can_replace_effective_selected_style() {
    let config = parse_config(
        r##"
        [ui.process_table.row]
        selected = { fg = "#FFFFFF", sub_modifier = "REVERSED", add_modifier = "BOLD" }
        "##,
    )
    .unwrap();

    assert_eq!(
        config.ui.process_table.row.selected,
        Style::new().fg(tailwind::WHITE).bold().not_reversed()
    );
}

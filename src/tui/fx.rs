use std::time::Instant;

use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::fx::effect_fn_buf;
use tachyonfx::{Effect, color_from_hsl, color_to_hsl};

use super::color_cycle::{ColorCycle, RepeatingColorCycle, RepeatingCycle};

/// I have copied this code without shame and regret from here: https://github.com/junkdog/exabind/blob/main/core/src/fx/effect.rs#L2
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UniqueEffectId {
    #[default]
    SearchStarted,
    SearchCompleted,
}
pub fn selected_category(base_color: Color, area: Rect) -> Effect {
    let color_cycle = select_category_color_cycle(base_color, 1);

    let effect = effect_fn_buf(Instant::now(), u32::MAX, move |started_at, ctx, buf| {
        let elapsed = started_at.elapsed().as_secs_f32();

        // speed n cells/s
        let idx = (elapsed * 30.0) as usize;

        let area = ctx.area;

        let mut update_cell = |(x, y): (u16, u16), idx: usize| {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_fg(*color_cycle.color_at(idx));
            }
        };

        (area.x..area.right()).enumerate().for_each(|(i, x)| {
            update_cell((x, area.y), idx + i);
        });

        let cell_idx_offset = area.width as usize;
        (area.y + 1..area.bottom() - 1)
            .enumerate()
            .for_each(|(i, y)| {
                update_cell((area.right() - 1, y), idx + i + cell_idx_offset);
            });

        let cell_idx_offset = cell_idx_offset + area.height.saturating_sub(2) as usize;
        (area.x..area.right()).rev().enumerate().for_each(|(i, x)| {
            update_cell((x, area.bottom() - 1), idx + i + cell_idx_offset);
        });

        let cell_idx_offset = cell_idx_offset + area.width as usize;
        (area.y + 1..area.bottom())
            .rev()
            .enumerate()
            .for_each(|(i, y)| {
                update_cell((area.x, y), idx + i + cell_idx_offset);
            });
    });

    effect.with_area(area)
}

fn select_category_color_cycle(
    base_color: Color,
    length_multiplier: usize,
) -> ColorCycle<RepeatingCycle> {
    let color_step: usize = 7 * length_multiplier;

    let (h, s, l) = color_to_hsl(&base_color);

    let color_l = color_from_hsl(h, s, 80.0);
    let color_d = color_from_hsl(h, s, 40.0);

    RepeatingColorCycle::new(
        base_color,
        &[
            (4 * length_multiplier, color_d),
            (2 * length_multiplier, color_l),
            (
                4 * length_multiplier,
                color_from_hsl((h - 25.0) % 360.0, s, (l + 10.0).min(100.0)),
            ),
            (
                color_step,
                color_from_hsl(h, (s - 20.0).max(0.0), (l + 10.0).min(100.0)),
            ),
            (
                color_step,
                color_from_hsl((h + 25.0) % 360.0, s, (l + 10.0).min(100.0)),
            ),
            (
                color_step,
                color_from_hsl(h, (s + 20.0).max(0.0), (l + 10.0).min(100.0)),
            ),
        ],
    )
}

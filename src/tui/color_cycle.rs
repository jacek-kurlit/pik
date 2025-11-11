/// I have copied this code from here: https://github.com/junkdog/exabind/blob/main/core/src/color_cycle.rs
use ratatui::prelude::Color;
use tachyonfx::Interpolatable;

pub trait IndexResolver<T: Clone> {
    fn resolve(idx: usize, data: &[T]) -> &T;
}

#[derive(Clone, Debug)]
pub struct PingPongCycle;
#[derive(Clone, Debug)]
pub struct RepeatingCycle;

#[derive(Clone, Debug)]
pub struct ColorCycle<T: IndexResolver<Color>> {
    colors: Vec<Color>,
    _marker: std::marker::PhantomData<T>,
}

impl IndexResolver<Color> for PingPongCycle {
    fn resolve(idx: usize, data: &[Color]) -> &Color {
        let dbl_idx = idx % (2 * data.len());
        let final_index = if dbl_idx < data.len() {
            dbl_idx
        } else {
            2 * data.len() - 1 - dbl_idx
        };

        data.get(final_index)
            .expect("ColorCycle: index out of bounds")
    }
}

impl IndexResolver<Color> for RepeatingCycle {
    fn resolve(idx: usize, data: &[Color]) -> &Color {
        data.get(idx % data.len())
            .expect("ColorCycle: index out of bounds")
    }
}

pub type PingPongColorCycle = ColorCycle<PingPongCycle>;
pub type RepeatingColorCycle = ColorCycle<RepeatingCycle>;

impl<T> ColorCycle<T>
where
    T: IndexResolver<Color>,
{
    pub fn new(initial_color: Color, colors: &[(usize, Color)]) -> Self {
        let mut gradient = vec![initial_color];
        colors
            .iter()
            .fold((0, initial_color), |(_, prev_color), (len, color)| {
                (0..=*len).for_each(|i| {
                    let color = prev_color.lerp(color, i as f32 / *len as f32);
                    gradient.push(color);
                });
                gradient.push(*color);
                (*len, *color)
            });

        Self {
            colors: gradient,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn color_at(&self, idx: usize) -> &Color {
        T::resolve(idx, &self.colors)
    }
}

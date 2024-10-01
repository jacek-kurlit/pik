use ratatui::Viewport;

use crate::{
    args::{CliArgs, ScreenSizeOptions},
    config::{AppConfig, ScreenSize},
    processes::FilterOptions,
};

pub struct AppSettings {
    pub viewport: Viewport,
    pub filter_opions: FilterOptions,
}

impl AppSettings {
    pub fn from(config: AppConfig, cli_args: &CliArgs) -> Self {
        Self {
            viewport: prefer_override(config.screen_size, cli_args.screen_size),
            filter_opions: FilterOptions {
                ignore_threads: !cli_args.include_threads_processes,
                include_all_processes: cli_args.include_other_users_processes,
            },
        }
    }
}

fn prefer_override<V, C, A>(config_value: C, override_opt: Option<A>) -> V
where
    C: Into<V>,
    A: Into<V>,
{
    match override_opt {
        Some(overidden_value) => overidden_value.into(),
        None => config_value.into(),
    }
}

impl From<ScreenSize> for Viewport {
    fn from(ss: ScreenSize) -> Self {
        match ss {
            ScreenSize::Fullscreen => Viewport::Fullscreen,
            ScreenSize::Height(height) => Viewport::Inline(height),
        }
    }
}

impl From<ScreenSizeOptions> for Viewport {
    fn from(ss: ScreenSizeOptions) -> Self {
        match (ss.fullscreen, ss.height) {
            (true, _) => Viewport::Fullscreen,
            (_, height) => Viewport::Inline(height),
        }
    }
}

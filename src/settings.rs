use ratatui::Viewport;

use crate::{
    args::{CliArgs, ScreenSizeOptions},
    config::{AppConfig, ScreenSize},
    processes::FilterOptions,
};

#[derive(Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_convert_screen_size_to_viewport() {
        assert_eq!(Viewport::from(ScreenSize::Fullscreen), Viewport::Fullscreen);
        assert_eq!(Viewport::from(ScreenSize::Height(25)), Viewport::Inline(25));
    }

    #[test]
    fn should_convert_screen_size_options_to_viewport() {
        assert_eq!(
            Viewport::from(ScreenSizeOptions {
                fullscreen: true,
                height: 25
            }),
            Viewport::Fullscreen
        );
        assert_eq!(
            Viewport::from(ScreenSizeOptions {
                fullscreen: false,
                height: 25
            }),
            Viewport::Inline(25)
        );
    }

    #[test]
    fn should_create_settings() {
        let config = AppConfig::default();
        let cli_args = CliArgs {
            query: "".to_string(),
            include_threads_processes: true,
            include_other_users_processes: true,
            screen_size: None,
        };
        let settings = AppSettings::from(config, &cli_args);
        assert_eq!(
            settings,
            AppSettings {
                viewport: Viewport::Inline(25),
                filter_opions: FilterOptions {
                    ignore_threads: false,
                    include_all_processes: true
                }
            }
        );
    }

    #[test]
    fn should_prefer_cli_args_screen_size() {
        let config = AppConfig {
            screen_size: ScreenSize::Height(40),
        };
        let cli_args = CliArgs {
            screen_size: Some(ScreenSizeOptions {
                fullscreen: true,
                height: 25,
            }),
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, &cli_args);
        assert_eq!(settings.viewport, Viewport::Fullscreen);
    }

    fn some_cli_args() -> CliArgs {
        CliArgs {
            query: "".to_string(),
            include_threads_processes: true,
            include_other_users_processes: true,
            screen_size: None,
        }
    }
}

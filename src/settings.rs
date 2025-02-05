use ratatui::Viewport;

use crate::{
    args::{CliArgs, ScreenSizeOptions},
    config::{AppConfig, ScreenSize},
    processes::IgnoreOptions,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AppSettings {
    pub query: String,
    pub viewport: Viewport,
    pub filter_opions: IgnoreOptions,
    pub use_icons: bool,
}

impl AppSettings {
    pub fn from(config: AppConfig, cli_args: CliArgs) -> Self {
        Self {
            query: cli_args.query,
            viewport: prefer_override(config.screen_size, cli_args.screen_size),
            filter_opions: IgnoreOptions {
                ignore_threads: !cli_args.ignore.include_threads_processes,
                ignore_other_users: !cli_args.ignore.include_other_users_processes,
                paths: cli_args.ignore.paths,
            },
            use_icons: config.use_icons,
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

    use crate::args::{self};

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
            screen_size: None,
            ignore: args::IgnoreOptions {
                include_threads_processes: true,
                include_other_users_processes: true,
                paths: vec![],
            },
        };
        let settings = AppSettings::from(config, cli_args);
        assert_eq!(
            settings,
            AppSettings {
                query: "".into(),
                viewport: Viewport::Inline(25),
                use_icons: false,
                filter_opions: IgnoreOptions {
                    ignore_threads: false,
                    ignore_other_users: false,
                    paths: vec![]
                }
            }
        );
    }

    #[test]
    fn should_prefer_cli_args_screen_size() {
        let config = AppConfig {
            screen_size: ScreenSize::Height(40),
            ..Default::default()
        };
        let cli_args = CliArgs {
            screen_size: Some(ScreenSizeOptions {
                fullscreen: true,
                height: 25,
            }),
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, cli_args);
        assert_eq!(settings.viewport, Viewport::Fullscreen);
    }

    fn some_cli_args() -> CliArgs {
        CliArgs {
            query: "".to_string(),
            screen_size: None,
            ignore: Default::default(),
        }
    }
}

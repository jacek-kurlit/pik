use ratatui::Viewport;

use crate::{
    args::{CliArgs, ScreenSizeOptions},
    config::{AppConfig, ScreenSize, keymappings::KeyMappings, ui::UIConfig},
    processes::IgnoreOptions,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AppSettings {
    pub query: String,
    pub viewport: Viewport,
    pub filter_opions: IgnoreOptions,
    pub ui_config: UIConfig,
    pub key_mappings: KeyMappings,
}

impl AppSettings {
    pub fn from(config: AppConfig, cli_args: CliArgs) -> Self {
        Self {
            query: cli_args.query,
            viewport: prefer_override(config.screen_size, cli_args.screen_size),
            filter_opions: IgnoreOptions {
                ignore_threads: prefer_override(
                    config.ignore.threads,
                    cli_args.ignore.ignore_thread_processes,
                ),
                ignore_other_users: prefer_override(
                    config.ignore.other_users,
                    cli_args.ignore.ignore_other_users_processes,
                ),
                paths: prefer_override(config.ignore.paths, cli_args.ignore.paths),
            },
            ui_config: config.ui,
            key_mappings: config.key_mappings.unwrap_or_default(),
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
    use regex::Regex;

    use crate::{
        args::{self},
        config::IgnoreConfig,
    };

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
                ignore_thread_processes: Some(false),
                ignore_other_users_processes: Some(false),
                paths: None,
            },
        };
        let settings = AppSettings::from(config, cli_args);
        assert_eq!(
            settings,
            AppSettings {
                query: "".into(),
                viewport: Viewport::Inline(25),
                filter_opions: IgnoreOptions {
                    ignore_threads: false,
                    ignore_other_users: false,
                    paths: vec![]
                },
                ui_config: UIConfig::default(),
                key_mappings: KeyMappings::preconfigured_mappings(),
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

    #[test]
    fn should_prefer_config_ignore_threads_when_cli_args_is_none() {
        let config = AppConfig {
            ignore: IgnoreConfig {
                threads: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let cli_args = CliArgs {
            ignore: args::IgnoreOptions {
                ignore_thread_processes: None,
                ..Default::default()
            },
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, cli_args);
        assert!(settings.filter_opions.ignore_threads);
    }

    #[test]
    fn should_prefer_cli_args_ignore_threads_when_flag_was_set() {
        let config = AppConfig {
            ignore: IgnoreConfig {
                threads: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let cli_args = CliArgs {
            ignore: args::IgnoreOptions {
                ignore_thread_processes: Some(true),
                ..Default::default()
            },
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, cli_args);
        assert!(settings.filter_opions.ignore_threads);
    }

    #[test]
    fn should_prefer_config_ignore_other_users_when_cli_args_is_none() {
        let config = AppConfig {
            ignore: IgnoreConfig {
                other_users: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let cli_args = CliArgs {
            ignore: args::IgnoreOptions {
                ignore_other_users_processes: None,
                ..Default::default()
            },
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, cli_args);
        assert!(settings.filter_opions.ignore_other_users);
    }

    #[test]
    fn should_prefer_cli_args_ignore_other_users_when_flag_was_set() {
        let config = AppConfig {
            ignore: IgnoreConfig {
                other_users: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let cli_args = CliArgs {
            ignore: args::IgnoreOptions {
                ignore_other_users_processes: Some(true),
                ..Default::default()
            },
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, cli_args);
        assert!(settings.filter_opions.ignore_other_users);
    }

    #[test]
    fn should_prefer_config_ignored_paths_when_cli_args_was_not_set() {
        let config = AppConfig {
            ignore: IgnoreConfig {
                paths: vec![Regex::new("/user/*").unwrap()],
                ..Default::default()
            },
            ..Default::default()
        };
        let cli_args = CliArgs {
            ignore: args::IgnoreOptions {
                paths: None,
                ..Default::default()
            },
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, cli_args);
        assert_eq!(settings.filter_opions.paths.len(), 1);
        assert_eq!(settings.filter_opions.paths[0].as_str(), "/user/*");
    }

    #[test]
    fn should_prefer_cli_args_ignored_paths() {
        let config = AppConfig {
            ignore: IgnoreConfig {
                paths: vec![Regex::new("/user/*").unwrap()],
                ..Default::default()
            },
            ..Default::default()
        };
        let cli_args = CliArgs {
            ignore: args::IgnoreOptions {
                paths: Some(vec![Regex::new("/*").unwrap()]),
                ..Default::default()
            },
            ..some_cli_args()
        };
        let settings = AppSettings::from(config, cli_args);
        assert_eq!(settings.filter_opions.paths.len(), 1);
        assert_eq!(settings.filter_opions.paths[0].as_str(), "/*");
    }

    fn some_cli_args() -> CliArgs {
        CliArgs {
            query: "".to_string(),
            screen_size: None,
            ignore: Default::default(),
        }
    }
}

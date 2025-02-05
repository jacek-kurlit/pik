use anyhow::{Context, Result};

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
    let raw_toml = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to load config from file: {:?}", path))?;
    toml::from_str(&raw_toml)
        .with_context(|| format!("Failed to deserialize config from file: {:?}", path))
}

use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub screen_size: ScreenSize,
    #[serde(default)]
    pub use_icons: bool,
    #[serde(default)]
    pub ignore: IgnoreConfig,
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
#[serde(rename_all = "camelCase")]
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

    use super::*;

    #[test]
    fn should_deserialize_empty_configuration() {
        let default_settings = toml::from_str("");
        assert_eq!(default_settings, Ok(AppConfig::default()));
        // ensure what actual defaults are
        assert_eq!(
            default_settings,
            Ok(AppConfig {
                screen_size: ScreenSize::Height(DEFAULT_SCREEN_SIZE),
                use_icons: false,
                ignore: IgnoreConfig {
                    paths: vec![],
                    other_users: true,
                    threads: true
                }
            })
        );
    }

    #[test]
    fn should_allow_to_override_defaults() {
        let default_settings: AppConfig = toml::from_str(
            r#"
            screen_size = "fullscreen"
            use_icons = true
            [ignore]
            paths=["/usr/*"]
            other_users = false
            threads = false
            "#,
        )
        .unwrap();
        assert_eq!(
            default_settings,
            AppConfig {
                screen_size: ScreenSize::Fullscreen,
                use_icons: true,
                ignore: IgnoreConfig {
                    paths: vec![Regex::new("/usr/*").unwrap()],
                    other_users: false,
                    threads: false
                }
            }
        );
    }
}

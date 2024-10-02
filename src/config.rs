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

use serde::Deserialize;

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub screen_size: ScreenSize,
}

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
    }

    #[test]
    fn should_allow_to_override_defaults() {
        let default_settings: AppConfig = toml::from_str(
            r#"
            screen_size = "fullscreen"
            "#,
        )
        .unwrap();
        assert_eq!(
            default_settings,
            AppConfig {
                screen_size: ScreenSize::Fullscreen
            }
        );
    }
}

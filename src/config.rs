use anyhow::{Context, Result};
use config::File;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct AppConfig {
    pub include_threads_processes: bool,
    pub include_other_users_processes: bool,
    pub screen_size: ScreenSize,
}

#[derive(Debug, Deserialize)]
pub enum ScreenSize {
    Fullscreen,
    Height(u16),
}

pub fn load_app_config() -> Result<AppConfig> {
    let config_path = directories::ProjectDirs::from("", "", "pik")
        .map(|dirs| dirs.project_path().join("config.toml"))
        .filter(|path| path.exists());

    match config_path {
        Some(path) => load_config_from_file(&path),
        None => Ok(AppConfig::default()),
    }
}

fn load_config_from_file(path: &std::path::PathBuf) -> Result<AppConfig> {
    config::Config::builder()
        .add_source(File::from(path.as_path()))
        .build()
        .with_context(|| format!("Failed to load config from file: {:?}", path))?
        .try_deserialize::<AppConfig>()
        .with_context(|| format!("Failed to deserialize config from file: {:?}", path))
}

pub const DEFAULT_SCREEN_SIZE: u16 = 20;

impl Default for ScreenSize {
    fn default() -> Self {
        ScreenSize::Height(DEFAULT_SCREEN_SIZE)
    }
}

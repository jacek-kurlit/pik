use anyhow::{Context, Result};
use config::File;
use serde::Deserialize;

use crate::args::CliArgs;

#[derive(Debug, Default, Deserialize)]
pub struct AppConfig {
    pub include_threads_processes: bool,
    pub include_other_users_processes: bool,
    pub screen_size: ScreenSize,
}

#[derive(Debug, Deserialize, Clone, Copy)]
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

impl AppConfig {
    //TODO: add tests
    pub fn override_with_args(&mut self, args: &CliArgs) {
        self.include_threads_processes = args
            .include_threads_processes
            .unwrap_or(self.include_threads_processes);
        self.include_other_users_processes = args
            .include_other_users_processes
            .unwrap_or(self.include_other_users_processes);
        self.screen_size = match &args.screen_size {
            Some(screen_options) => match (screen_options.fullscreen, screen_options.height) {
                (true, _) => ScreenSize::Fullscreen,
                (_, height) => ScreenSize::Height(height),
            },
            None => self.screen_size,
        };
    }
}

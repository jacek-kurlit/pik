use clap::{Args, Parser};
use regex::Regex;

use crate::config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = Some("Pik is a simple TUI tool for searching and killing processes in interactive way."))]
pub struct CliArgs {
    #[clap(
        default_value = "",
        help = r#"Query string for searching processes.
        You may use special prefix for different kind of search:
        - :<port> - search by port, i.e ':8080'
        - /<path> - search by command path, i.e. '/home/user/bin'
        - -<arg> - search by argument, i.e. '-i'
        If no prefix is given search will be done by process name"#
    )]
    pub query: String,
    #[command(flatten)]
    pub ignore: IgnoreOptions,
    #[command(flatten)]
    pub screen_size: Option<ScreenSizeOptions>,
}

#[derive(Args, Debug, Clone, Copy)]
#[group(required = false, multiple = false)]
pub struct ScreenSizeOptions {
    /// Start pik in fullscreen mode
    #[arg(short = 'F', long, default_value_t = false)]
    pub fullscreen: bool,
    /// Number of lines of the screen pik will use
    #[arg(short = 'H', long, default_value_t = config::DEFAULT_SCREEN_SIZE)]
    pub height: u16,
}

#[derive(Args, Debug, Clone, Default)]
#[group(required = false, multiple = true, id = "Ignored Options")]
/// Ignored Options
pub struct IgnoreOptions {
    /// Allows to include/exclude thread processes (linux). If not set all threads processes are ignored
    #[arg(
        help_heading = "Ignore Options",
        short = 't',
        long,
        default_value = None
    )]
    pub ignore_thread_processes: Option<bool>,
    /// Allows to include/exclude processes owned by other users. If not set only current user processes are
    /// shown
    #[arg(
        help_heading = "Ignore Options",
        short = 'o',
        long,
        default_value = None
    )]
    pub ignore_other_users_processes: Option<bool>,
    /// Ignore processes that cmd path matches any of provided regexes.
    /// List of regex example: -p "/path/.*" -p "/other/.*"
    #[arg(help_heading = "Ignore Options", short = 'p', long = "ignore-path")]
    pub paths: Option<Vec<Regex>>,
}

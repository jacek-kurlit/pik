use clap::{Args, Parser};

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
    /// On linux threads can be listed as processes which are ignored by default. This flag allows to include them
    #[arg(short = 't', long)]
    pub include_threads_processes: Option<bool>,
    /// By default pik shows only proceseses owned by current user. This flag allows to show all processes
    #[arg(short = 'a', long)]
    pub include_other_users_processes: Option<bool>,
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

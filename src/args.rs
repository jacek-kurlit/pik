use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = Some("Pik is a simple TUI tool for searching and killing processes in interactive way."))]
pub struct Args {
    #[clap(
        default_value = "",
        help = r#"Query string for searching processes. By default, all processes are searched.
        You may use special prefix for different kind of search:
        - :<port> - search by port, i.e ':8080'
        - /<path> - search by command path, i.e. '/home/user/bin'
        - -<arg> - search by argument, i.e. '-i'
        If no prefix is given search will be done by process name"#
    )]
    pub query: String,
    /// On linux threads can be listed as processes which are ignored by default. This option allows to include them
    #[arg(short, long, default_value_t = false)]
    pub threads_processes: bool,
    /// By default pik shows only proceseses owned by current user. This option allows to show all processes
    #[arg(short, long, default_value_t = false)]
    pub all_processes: bool,
}

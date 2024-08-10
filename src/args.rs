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
}

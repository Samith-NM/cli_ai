use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "cli_ai", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run {
        cli_ai: String,

        #[arg(last = true)]
        args: Vec<String>,

        #[arg(long, default_value_t = 5)]
        context: usize,

        #[arg(long)]
        mock: bool,

        #[arg(long, default_value = "cli_ai_bugreports")]
        report_dir: String,

        #[arg(long)]
        no_reports: bool,

        #[arg(long, default_value_t = 15)]
        timeout: u64,
    },
}

use clap::{Parser, Subcommand};

pub struct Cli;

impl Cli {
    pub fn run(&self) -> CliProgram {
        CliProgram::parse()
    }
}

#[derive(Parser)]
#[command(author, version, about)]
pub struct CliProgram {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    #[command(
        about = "Take bookmarks, and prepare them to synchronization, by saving in process database"
    )]
    Prepare {},
    #[command(about = "Synchronize all pending bookmarks")]
    Synchronize {
        #[arg(
            long,
            value_name = "FILTER_EXPRESSION",
            help = "Options for --match-filter (https://github.com/ytdl-org/youtube-dl/blob/master/README.md#video-selection)"
        )]
        filter: Option<String>,
    },
    #[command(about = "Prints failed processes")]
    Failed {
        #[arg(short, long, help = "List only failed YouTube ids without decorations")]
        short: bool,
    },
}

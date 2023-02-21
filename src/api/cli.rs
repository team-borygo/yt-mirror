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
  #[command(about = "Take bookmarks, and prepare them to synchronization, by saving in process database")]
  Prepare {
    #[arg(short, long, value_name = "FILE", help = "Location for processes database (created automatically if doesn't exist)")]
    processes: String,

    #[arg(short, long, value_name = "FILE", help = "Location of browser database (see README for more details)")]
    bookmarks: String,
  },
  #[command(about = "Synchronize all pending bookmarks")]
  Synchronize {
    #[arg(short, long, value_name = "FILE", help = "Location for processes database (created automatically if doesn't exist)")]
    processes: String,

    #[arg(short, long, value_name = "DIRECTORY", help = "Path to a directory into which music files will be downloaded")]
    target: String,

    #[arg(long, value_name = "DIRECTORY", help = "Path to a directory in which temporary files will be stored", default_value_t = String::from("/tmp"))]
    tmp: String,

    #[arg(long, value_name = "FILTER_EXPRESSION", help = "Options for --match-filter (https://github.com/ytdl-org/youtube-dl/blob/master/README.md#video-selection)")]
    filter: Option<String>,
  },
  #[command(about = "Prints failed processes")]
  Failed {
    #[arg(short, long, value_name = "FILE", help = "Location for processes database (created automatically if doesn't exist)")]
    processes: String,

    #[arg(short, long, help = "List only failed YouTube ids without decorations")]
    short: bool,
  },
}
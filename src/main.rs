use crate::api::cli::Cli;

mod api;
mod library;
mod types;

fn main() {
    println!("Hello, world!");

    let cli = Cli {};
    let program = cli.run();
}

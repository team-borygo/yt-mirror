use crate::api::cli::Cli;

mod api;

fn main() {
    println!("Hello, world!");

    let cli = Cli {};
    let program = cli.run();
}

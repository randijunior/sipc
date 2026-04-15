use clap::Parser;

use crate::cli::Cli;

mod cli;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    Ok(())
}

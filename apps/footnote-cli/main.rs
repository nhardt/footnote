mod commands;

use clap::Parser;
use commands::{execute, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    execute(cli).await
}

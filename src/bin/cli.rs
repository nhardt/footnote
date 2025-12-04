use clap::Parser;
use footnote::cli::commands::{Cli, execute};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    execute(cli).await
}

use clap::Parser;
use footnote::cli::commands::{execute, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    execute(cli).await
}

use crate::model::vault::Vault;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "footnote")]
#[command(about = "A CLI tool for p2p sync and share", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
}

#[derive(Subcommand)]
pub enum VaultAction {
    CreatePrimary { device_name: String },
    CreateSecondary { device_name: String },
}

pub async fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Vault { action } => match action {
            VaultAction::CreatePrimary { device_name } => vault_create_primary(device_name),
            VaultAction::CreateSecondary { device_name } => vault_create_secondary(device_name),
        },
    }
}

fn vault_create_primary(device_name: String) -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    Vault::create_primary(vault_path, &device_name)?;
    let output = serde_json::json!({
        "result": "success"
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

fn vault_create_secondary(device_name: String) -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    Vault::create_secondary(vault_path, &device_name)?;
    let output = serde_json::json!({
        "result": "success"
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

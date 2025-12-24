use crate::service::join_service::JoinService;
use crate::{model::vault::Vault, service::join_service::JoinEvent};
use clap::{Parser, Subcommand};
use futures::future::Join;

#[derive(Parser)]
#[command(name = "footnote")]
#[command(about = "A CLI tool for p2p sync and share", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Base commands to create and join vaults
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
}

#[derive(Subcommand)]
pub enum VaultAction {
    /// START HERE. Initialize a vault on the device that is on the most often
    CreatePrimary {
        /// this username is stored locally and is part of your signed
        /// contact record, if you choose to share footnotes with friends
        username: String,
        /// your name of this device, "desktop", "laptop"
        device_name: String,
    },
    /// After establishing a primary vault on your main computer, secondary
    /// devices can be added
    CreateSecondary {
        /// colloquial name of this device
        device_name: String,
    },
    /// Call device create on the primary device. The device will create a join code,
    /// then being listening for the a device to join. The joining device will send
    /// the one time. If contact can be established, a new contact record will be
    /// minted on the primary containing the joined device.
    JoinListen {},

    /// When device create is called on the primary, it will output a connection
    /// string. The connection string should be passed in here.
    Join { connect_string: String },
}

pub async fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Vault { action } => match action {
            VaultAction::CreatePrimary {
                username,
                device_name,
            } => vault_create_primary(username, device_name),
            VaultAction::CreateSecondary { device_name } => vault_create_secondary(device_name),
            VaultAction::JoinListen {} => vault_join_listen().await,
            VaultAction::Join { connect_string } => vault_join(connect_string).await,
        },
    }
}

fn vault_create_primary(username: String, device_name: String) -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    Vault::create_primary(vault_path, &username, &device_name)?;
    let output = serde_json::json!({
        "result": "success"
    });
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}

fn vault_create_secondary(device_name: String) -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    Vault::create_secondary(vault_path, &device_name)?;
    let output = serde_json::json!({
        "result": "success"
    });
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}

async fn vault_join_listen() -> anyhow::Result<()> {
    let vault = Vault::new(std::env::current_dir()?)?;
    let mut rx = JoinService::listen(&vault).await?;

    while let Some(event) = rx.recv().await {
        match event {
            JoinEvent::Listening { join_url } => {
                println!(
                    "{}",
                    serde_json::json!(
                        {
                            "event": "listening",
                            "join_url": join_url
                        }
                    )
                );
            }
            JoinEvent::Success => {
                println!(
                    "{}",
                    serde_json::json!(
                        {
                            "event": "success",
                        }
                    )
                );
                break;
            }
            JoinEvent::Error(detail) => {
                println!(
                    "{}",
                    serde_json::json!(
                        {
                            "event": "error",
                            "detail": detail
                        }
                    )
                );
                break;
            }
        }
    }

    Ok(())
}

async fn vault_join(connection_string: String) -> anyhow::Result<()> {
    let vault = Vault::new(std::env::current_dir()?)?;
    JoinService::join(&vault, &connection_string).await?;
    println!(
        "{}",
        serde_json::json!(
            {
                "event": "join.success",
                "detail": ""
            }
        )
    );
    Ok(())
}

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
    CreatePrimary {
        /// this username is stored in your user record and will be exported as
        /// part of your signed contact record
        username: String,
        /// colloquial name of this device
        device_name: String,
    },
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
            VaultAction::JoinListen {} => vault_join_listen(),
            VaultAction::Join { connect_string } => vault_join(connect_string),
        },
    }
}

fn vault_create_primary(username: String, device_name: String) -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    Vault::create_primary(vault_path, &username, &device_name)?;
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

async fn vault_join_listen() -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    let vault = Vault::new(vault_path)?;
    let (mut rx, cancel_token) = vault.join_listen().await?;
    while let Some(event) = rx.recv().await {
        match event {
            ListenEvent::Started { endpoint_id } => {
                eprintln!("Listening for connection to: {}", endpoint_id);
                break;
            }
            ListenEvent::Error(e) => {
                eprintln!("Error starting listener: {}", e);
                return Err(anyhow::anyhow!("Failed to start listener: {}", e));
            }
            _ => {}
        }
    }

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                match event {
                    ListenEvent::Received { from: _ } => {
                        eprintln!("ListenEvent::Received");
                    }
                    ListenEvent::Error(e) => {
                        eprintln!("ListenEvent::Error: {}", e);
                    }
                    ListenEvent::Stopped => {
                        eprintln!("ListenEvent::Stopped");
                        break;
                    }
                    _ => {}
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\nShut down requested...");
                cancel_token.cancel();
                break;
            }
        }
    }

    let output = serde_json::json!({
        "result": "success"
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

fn vault_join(_connection_string: String) -> anyhow::Result<()> {
    Ok(())
}

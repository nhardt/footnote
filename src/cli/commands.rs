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
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },
    Contact {
        #[command(subcommand)]
        action: ContactAction,
    },
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },
}

#[derive(Subcommand)]
pub enum VaultAction {
    Create {
        path: Option<std::path::PathBuf>,

        #[arg(long)]
        username: Option<String>,

        #[arg(long)]
        device_name: Option<String>,
    },
    Join {
        device_name: String,
        url: String,
        path: Option<std::path::PathBuf>,
    },
    ListenDevice {
        remote_url: String,

        #[arg(long)]
        device_name: String,
    },
    ListenFiles,
}

#[derive(Subcommand)]
pub enum ContactAction {
    Import {
        file_path: String,

        #[arg(long)]
        petname: String,
    },
    Read,
    Delete {
        user_name: String,
    },
    Export {
        user_name: String,
    },
    Share {
        #[arg(long)]
        user: String,

        #[arg(long)]
        device: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DeviceAction {
    Create {
        #[command(subcommand)]
        mode: Option<CreateMode>,
    },
    Delete {
        user_name: String,
        device_name: String,
    },
    Sync {
        #[arg(long)]
        user: Option<String>,

        #[arg(long)]
        device: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum NoteAction {
    Create { path: String, body: String },
    Update { uuid: String, body: String },
    Delete { uuid: String },
    // Mark note for sharing with <petname>
    // Share {
    //     uuid: String,
    //     petname: String,
    // },
    // not sure if we will need this, or if we want ShareAdd/ShareRemove or
    // Share(vector<String>)
}

pub async fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Vault { action } => match action {
            VaultAction::Create {
                path,
                username,
                device_name,
            } => {
                use crate::model::Vault;
                let vault_path = path.unwrap_or_else(|| {
                    std::env::current_dir().expect("Failed to get current directory")
                });
                let username = username.as_deref().unwrap_or("me");
                let device_name = device_name.as_deref().unwrap_or("primary");

                let vault = Vault::create(vault_path, username, device_name)?;

                // Output vault info as JSON for CLI
                let output = serde_json::json!({
                    "vault_path": vault.path().display().to_string(),
                    "master_public_key": vault.master_public_key()?,
                    "device_name": device_name,
                    "device_endpoint_id": vault.device_endpoint_id()?,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);

                Ok(())
            }
            VaultAction::Join {
                device_name,
                url,
                path,
            } => {
                let vault_path = path.unwrap_or_else(|| {
                    std::env::current_dir().expect("Failed to get current directory")
                });
                crate::core::device::create_remote(&vault_path, &url, &device_name).await
            }
            VaultAction::Listen => {
                use crate::model::{ListenEvent, Vault};
                let vp = vault_path
                    .as_ref()
                    .expect("vault required for this command");

                // Open vault and start listening in background
                let vault = Vault::open(vp.clone())?;
                let (mut rx, cancel_token) = vault.listen_background().await?;

                // Wait for Started event and print header
                while let Some(event) = rx.recv().await {
                    match event {
                        ListenEvent::Started { endpoint_id } => {
                            println!("\n📡 Vault - Listening");
                            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                            println!("Endpoint ID: {}", endpoint_id);
                            println!("Ready to receive syncs and shares");
                            println!("\nPress Ctrl+C to stop listening");
                            println!();
                            break;
                        }
                        ListenEvent::Error(e) => {
                            eprintln!("Error starting listener: {}", e);
                            return Err(anyhow::anyhow!("Failed to start listener: {}", e));
                        }
                        _ => {}
                    }
                }

                // Handle events until Ctrl+C
                loop {
                    tokio::select! {
                        Some(event) = rx.recv() => {
                            match event {
                                ListenEvent::Received { from: _ } => {
                                    // Events are already printed by sync handler
                                }
                                ListenEvent::Error(e) => {
                                    eprintln!("Error: {}", e);
                                }
                                ListenEvent::Stopped => {
                                    break;
                                }
                                _ => {}
                            }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            println!("\nShutting down...");
                            cancel_token.cancel();
                            break;
                        }
                    }
                }

                Ok(())
            }
        },
        Commands::Trust { file_path, petname } => {
            let vp = vault_path
                .as_ref()
                .expect("vault required for this command");
            crate::core::user::import(vp, &file_path, &petname).await
        }
        Commands::User { action } => match action {
            ContactAction::Create { user_name } => crate::core::user::create(&user_name).await,
            ContactAction::Delete { user_name } => crate::core::user::delete(&user_name).await,
            ContactAction::Read => {
                let vp = vault_path
                    .as_ref()
                    .expect("vault required for this command");
                crate::core::user::read(vp).await
            }
            ContactAction::Export { user_name } => {
                let vp = vault_path
                    .as_ref()
                    .expect("vault required for this command");
                crate::core::user::export(vp, &user_name).await
            }
        },
        Commands::Device { action } => match action {
            DeviceAction::Delete {
                user_name,
                device_name,
            } => crate::core::device::delete(&user_name, &device_name).await,
            DeviceAction::Create { mode } => match mode {
                None => {
                    // Primary device: generate join URL and handle events
                    let vp = vault_path
                        .as_ref()
                        .expect("vault required for this command");
                    let mut rx = crate::core::device::create_primary(vp).await?;

                    while let Some(event) = rx.recv().await {
                        match event {
                            crate::core::device::DeviceAuthEvent::Listening { join_url } => {
                                println!("\n🔐 Device Authorization");
                                println!("Copy this URL to your new device:");
                                println!("  {}", join_url);
                                println!("\nWaiting for device to connect...");
                            }
                            crate::core::device::DeviceAuthEvent::Connecting => {
                                println!("✓ Device connecting...");
                            }
                            crate::core::device::DeviceAuthEvent::ReceivedRequest {
                                device_name,
                            } => {
                                println!("✓ Received request from: {}", device_name);
                            }
                            crate::core::device::DeviceAuthEvent::Verifying => {
                                println!("✓ Verifying...");
                            }
                            crate::core::device::DeviceAuthEvent::Success { device_name } => {
                                println!("✓ Success! Device '{}' has been authorized", device_name);
                                break;
                            }
                            crate::core::device::DeviceAuthEvent::Error(err) => {
                                println!("✗ Error: {}", err);
                                break;
                            }
                        }
                    }
                    Ok(())
                }
                Some(CreateMode::Remote {
                    remote_url,
                    device_name,
                }) => {
                    // Remote device: join using URL
                    let vp = vault_path
                        .as_ref()
                        .expect("vault required for this command");
                    crate::core::device::create_remote(vp, &remote_url, &device_name).await
                }
            },
        },
        Commands::Share { petname } => {
            let vp = vault_path
                .as_ref()
                .expect("vault required for this command");
            crate::model::contact::share(vp, petname.as_deref()).await
        }
    }
}

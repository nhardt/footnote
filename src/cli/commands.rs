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
    /// Initialize vault and create primary device
    Init {
        /// Path to create vault (defaults to current directory)
        path: Option<std::path::PathBuf>,

        /// Username for this identity (optional)
        #[arg(long)]
        username: Option<String>,

        /// Name for this device (optional)
        #[arg(long)]
        device_name: Option<String>,
    },
    /// Trust a user by importing their contact information
    Trust {
        /// Path to the contact file to import
        file_path: String,
        /// Petname for this user (what you call them locally)
        #[arg(long)]
        petname: String,
    },
    /// User management commands
    User {
        #[command(subcommand)]
        action: UserAction,
    },
    /// Device management commands
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },
    /// Mirror operations
    Mirror {
        #[command(subcommand)]
        action: MirrorAction,
    },
    /// Share documents with a trusted user
    Share {
        /// Petname of the user to share with (or omit to share with all)
        petname: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum UserAction {
    /// Create a new user
    Create { user_name: String },
    /// Delete a user
    Delete { user_name: String },
    /// Read and display all users and their devices
    Read,
    /// Export a user's contact information
    Export { user_name: String },
}

#[derive(Subcommand)]
pub enum DeviceAction {
    /// Delete a device
    Delete {
        user_name: String,
        device_name: String,
    },
    /// Create and authorize a new device (primary device generates join URL, or
    /// remote device joins)
    Create {
        #[command(subcommand)]
        mode: Option<CreateMode>,
    },
}

#[derive(Subcommand)]
pub enum CreateMode {
    /// Join from a remote device using a connection URL
    Remote {
        /// Connection URL from primary device (iroh://endpoint-id?token=xyz)
        remote_url: String,
        /// Name for this device
        #[arg(long)]
        device_name: String,
    },
}

#[derive(Subcommand)]
pub enum MirrorAction {
    /// Listen for incoming mirror connections
    Listen,
    /// Connect from a remote device using a connection URL
    From {
        /// Connection URL from primary device
        remote_url: String,
        /// Name for this device
        #[arg(long)]
        device_name: String,
    },
    /// Push mirror data
    Push {
        /// Optional user name
        #[arg(long)]
        user: Option<String>,
        /// Optional device name (requires user)
        #[arg(long)]
        device: Option<String>,
    },
}

/// Execute the CLI command
pub async fn execute(cli: Cli) -> anyhow::Result<()> {
    let needs_vault = match &cli.command {
        Commands::Init { .. } => false,
        Commands::Mirror { action } => match action {
            MirrorAction::From { .. } => false,
            _ => true,
        },
        Commands::Device { action } => match action {
            DeviceAction::Create {
                mode: Some(CreateMode::Remote { .. }),
            } => false,
            _ => true,
        },
        _ => true,
    };

    // Get vault path for commands that need it
    let vault_path = if needs_vault {
        let vp = crate::core::vault::get_vault_path()?;
        crate::core::vault::verify_vault_layout()?;
        Some(vp)
    } else {
        None
    };

    match cli.command {
        Commands::Init {
            path,
            username,
            device_name,
        } => crate::core::init::init(path, username.as_deref(), device_name.as_deref()).await,
        Commands::Trust { file_path, petname } => {
            let vp = vault_path
                .as_ref()
                .expect("vault required for this command");
            crate::core::user::import(vp, &file_path, &petname).await
        }
        Commands::User { action } => match action {
            UserAction::Create { user_name } => crate::core::user::create(&user_name).await,
            UserAction::Delete { user_name } => crate::core::user::delete(&user_name).await,
            UserAction::Read => {
                let vp = vault_path
                    .as_ref()
                    .expect("vault required for this command");
                crate::core::user::read(vp).await
            }
            UserAction::Export { user_name } => {
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
        Commands::Mirror { action } => match action {
            MirrorAction::Listen => {
                let vp = vault_path
                    .as_ref()
                    .expect("vault required for this command");
                crate::core::mirror::listen(vp).await
            }
            MirrorAction::From {
                remote_url,
                device_name,
            } => {
                let vp = vault_path
                    .as_ref()
                    .expect("vault required for this command");
                crate::core::device::create_remote(vp, &remote_url, &device_name).await
            }
            MirrorAction::Push { user, device } => {
                let vp = vault_path
                    .as_ref()
                    .expect("vault required for this command");
                crate::core::mirror::push(vp, user.as_deref(), device.as_deref()).await
            }
        },
        Commands::Share { petname } => {
            let vp = vault_path
                .as_ref()
                .expect("vault required for this command");
            crate::core::mirror::share(vp, petname.as_deref()).await
        }
    }
}

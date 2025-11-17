use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fieldnote")]
#[command(about = "A CLI tool for p2p sync and share", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize fieldnote vault and create default structure
    Init,
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
}

#[derive(Subcommand)]
pub enum UserAction {
    /// Create a new user
    Create { user_name: String },
    /// Delete a user
    Delete { user_name: String },
    /// Read and display all users and their devices
    Read,
}

#[derive(Subcommand)]
pub enum DeviceAction {
    /// Create a new device for a user
    Create {
        user_name: String,
        device_name: String,
    },
    /// Delete a device
    Delete {
        user_name: String,
        device_name: String,
    },
}

#[derive(Subcommand)]
pub enum MirrorAction {
    /// Listen for incoming mirror connections
    Listen,
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
    match cli.command {
        Commands::Init => crate::core::init::initialize().await,
        Commands::User { action } => match action {
            UserAction::Create { user_name } => {
                crate::core::user::create(&user_name).await
            }
            UserAction::Delete { user_name } => {
                crate::core::user::delete(&user_name).await
            }
            UserAction::Read => crate::core::user::read().await,
        },
        Commands::Device { action } => match action {
            DeviceAction::Create {
                user_name,
                device_name,
            } => crate::core::device::create(&user_name, &device_name).await,
            DeviceAction::Delete {
                user_name,
                device_name,
            } => crate::core::device::delete(&user_name, &device_name).await,
        },
        Commands::Mirror { action } => match action {
            MirrorAction::Listen => crate::core::mirror::listen().await,
            MirrorAction::Push { user, device } => {
                crate::core::mirror::push(user.as_deref(), device.as_deref()).await
            }
        },
    }
}

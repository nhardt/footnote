use clap::{Parser, Subcommand};

use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

use footnote_core::model::contact::Contact;
use footnote_core::model::note::Note;
use footnote_core::model::vault::Vault;
use footnote_core::service::join_service::{JoinEvent, JoinService};
use footnote_core::service::sync_service::SyncService;
use footnote_core::service::ALPN_SYNC;

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
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },
    Contact {
        #[command(subcommand)]
        action: ContactAction,
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
    /// Create a standalone vault ready to join an existing device group.
    /// After running this, use `service join-listen` to generate a join code.
    CreateStandalone {},

    /// look through vault for md files that don't have metadata or have
    /// duplicate ids
    Doctor {
        /// uniquify uuids
        #[arg(short, long, default_value_t = false)]
        fix: bool,
    },
}

#[derive(Subcommand)]
pub enum ServiceAction {
    /// Listen for a primary device to add this device to their group.
    /// The primary device will scan the QR code or enter the join URL,
    /// provide a name for this device, and send the complete user record.
    JoinListen {},

    /// Add a new device to your group. Provide the join URL from the listening
    /// device and a name for that device (e.g., "laptop", "phone").
    Join {
        connect_string: String,
        device_name: String,
    },

    /// Replicate is used for two devices owned by same person. One side
    /// listens, one side pushes. in general, if you are on a device and
    /// writing to it, you'll push your changes out to replicas when saving.
    /// if the device is listening, it will be pushed to.
    ReplicateListen {},

    /// ensure that the given device name, that you should already have joined
    /// to this vault, has the most recent copy of all local files
    Replicate { to_device_name: String },

    /// Share is used for devices owned by different people. One side
    /// listens, one side pushes. in general, a user might always leave their
    /// primary listening, or they may coordinate to receive files
    ShareListen {},

    /// share to the primary device for the user with the given nickname you
    /// have previously joined
    Share { to_nickname: String },
}

#[derive(Subcommand)]
pub enum ContactAction {
    /// Export your information as a sharable contact
    Export {},
    /// Record the contact details of a friend so you can publish your shared
    /// notes to them.
    Import { nickname: String, path: PathBuf },
    /// show all trusted contacts
    Read {},
}

#[derive(Subcommand)]
pub enum NoteAction {
    Create {
        path: PathBuf,
        content: String,
    },
    Update {
        path: PathBuf,
        content: String,
        #[arg(long, num_args = 1..)]
        share: Option<Vec<String>>,
    },
    Delete {
        path: PathBuf,
    },
}

pub async fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Vault { action } => match action {
            VaultAction::CreatePrimary {
                username,
                device_name,
            } => vault_create_primary(username, device_name),
            VaultAction::CreateStandalone {} => vault_create_standalone(),
            VaultAction::Doctor { fix } => vault_doctor(fix),
        },
        Commands::Service { action } => match action {
            ServiceAction::JoinListen {} => service_join_listen().await,
            ServiceAction::Join {
                connect_string,
                device_name,
            } => service_join(connect_string, device_name).await,
            ServiceAction::ReplicateListen {} => service_replicate_listen().await,
            ServiceAction::Replicate { to_device_name } => service_replicate(to_device_name).await,
            ServiceAction::ShareListen {} => service_share_listen().await,
            ServiceAction::Share { to_nickname } => service_share(to_nickname).await,
        },
        Commands::Note { action } => match action {
            NoteAction::Create { path, content } => note_create(&path, &content),
            NoteAction::Update {
                path,
                content,
                share,
            } => note_update(&path, &content, share),
            NoteAction::Delete { path } => note_delete(&path),
        },
        Commands::Contact { action } => match action {
            ContactAction::Export {} => contact_export(),
            ContactAction::Import { nickname, path } => contact_import(&nickname, &path),
            ContactAction::Read {} => contact_read(),
        },
    }
}

fn vault_create_primary(username: String, device_name: String) -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    Vault::create_primary(&vault_path, &username, &device_name)?;
    let output = serde_json::json!({
        "result": "success"
    });
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}

fn vault_create_standalone() -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    Vault::create_standalone(&vault_path)?;
    let output = serde_json::json!({
        "result": "success"
    });
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}

fn vault_doctor(fix: bool) -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    let vault = Vault::new(&vault_path)?;
    let diagnostics = vault.doctor(fix)?;
    for (u, m) in diagnostics {
        println!("{}:{}", u, m);
    }
    Ok(())
}

async fn service_join_listen() -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    let cancel_token = CancellationToken::new();
    let mut rx = JoinService::listen(&vault, cancel_token).await?;

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

async fn service_join(connection_string: String, device_name: String) -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    JoinService::join(&vault, &connection_string, &device_name).await?;
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

async fn service_replicate_listen() -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    let endpoint = vault.build_endpoint(ALPN_SYNC).await?;
    let cancel_token = CancellationToken::new();
    SyncService::listen(vault, endpoint, cancel_token).await?;
    Ok(())
}

async fn service_replicate(to_device_name: String) -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    let endpoint = vault.build_endpoint(ALPN_SYNC).await?;
    SyncService::mirror_to_device(&vault, endpoint, &to_device_name).await?;
    println!(
        "{}",
        serde_json::json!(
            {
                "event": "push.success",
                "detail": ""
            }
        )
    );
    Ok(())
}

async fn service_share_listen() -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    let endpoint = vault.build_endpoint(ALPN_SYNC).await?;
    let cancel_token = CancellationToken::new();
    SyncService::listen(vault, endpoint, cancel_token).await?;
    Ok(())
}

async fn service_share(to_nickname: String) -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    let endpoint = vault.build_endpoint(ALPN_SYNC).await?;
    SyncService::share_to_device(&vault, endpoint, &to_nickname).await?;
    println!(
        "{}",
        serde_json::json!(
            {
                "event": "push.success",
                "detail": ""
            }
        )
    );
    Ok(())
}

fn note_create(path: &Path, content: &str) -> anyhow::Result<()> {
    let note_path = std::env::current_dir()?.join(path);
    Note::create(&note_path, content)?;
    Ok(())
}

fn note_update(path: &Path, content: &str, shares: Option<Vec<String>>) -> anyhow::Result<()> {
    let note_path = std::env::current_dir()?.join(path);
    let mut n = Note::from_path(note_path, false)?;
    if let Some(updated_shares) = shares {
        n.frontmatter.share_with = updated_shares;
    }
    n.update(path, content)?;
    Ok(())
}

fn note_delete(path: &Path) -> anyhow::Result<()> {
    println!("unimpl");
    Ok(())
}

fn contact_export() -> anyhow::Result<()> {
    let vault_path = std::env::current_dir()?;
    // by running our user record through Contact we get file verification
    let mut my_contact_record = Contact::from_file(vault_path.join(".footnote").join("user.json"))?;
    my_contact_record.nickname.clear();
    println!("{}", my_contact_record.to_json_pretty()?);
    Ok(())
}

fn contact_import(nickname: &str, path: &Path) -> anyhow::Result<()> {
    let contact_path = std::env::current_dir()?
        .join(".footnote")
        .join("contacts")
        .join(format!("{}.json", nickname));
    let mut c = Contact::from_file(path)?;
    c.nickname = nickname.to_string();
    c.to_file(contact_path)?;
    Ok(())
}

fn contact_read() -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    let contacts = vault.contact_read()?;
    for contact in contacts {
        println!("{}:{}", contact.nickname, contact.username);
    }
    Ok(())
}

use std::path::{Path, PathBuf};

use crate::model::contact::Contact;
use crate::model::note::Note;
use crate::service::join_service::JoinService;
use crate::service::replica_service::{ReplicaEvent, ReplicaService};
use crate::service::share_service::ShareService;
use crate::{model::vault::Vault, service::join_service::JoinEvent};
use clap::{Parser, Subcommand};
use dioxus::html::g::to;
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
    /// After establishing a primary vault on your main computer, secondary
    /// devices can be added
    CreateSecondary {
        /// colloquial name of this device
        device_name: String,
    },
}

#[derive(Subcommand)]
pub enum ServiceAction {
    /// Call device create on the primary device. The device will create a join code,
    /// then being listening for the a device to join. The joining device will send
    /// the one time. If contact can be established, a new contact record will be
    /// minted on the primary containing the joined device.
    JoinListen {},

    /// When device create is called on the primary, it will output a connection
    /// string. The connection string should be passed in here.
    Join { connect_string: String },

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
}

#[derive(Subcommand)]
pub enum NoteAction {
    Create { path: PathBuf, content: String },
    Update { path: PathBuf, content: String },
    Delete { path: PathBuf },
}

pub async fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Vault { action } => match action {
            VaultAction::CreatePrimary {
                username,
                device_name,
            } => vault_create_primary(username, device_name),
            VaultAction::CreateSecondary { device_name } => vault_create_secondary(device_name),
        },
        Commands::Service { action } => match action {
            ServiceAction::JoinListen {} => service_join_listen().await,
            ServiceAction::Join { connect_string } => service_join(connect_string).await,
            ServiceAction::ReplicateListen {} => service_replicate_listen().await,
            ServiceAction::Replicate { to_device_name } => service_replicate(to_device_name).await,
            ServiceAction::ShareListen {} => service_share_listen().await,
            ServiceAction::Share { to_nickname } => service_share(to_nickname).await,
        },
        Commands::Note { action } => match action {
            NoteAction::Create { path, content } => note_create(&path, &content),
            NoteAction::Update { path, content } => note_update(&path, &content),
            NoteAction::Delete { path } => note_delete(&path),
        },
        Commands::Contact { action } => match action {
            ContactAction::Export {} => contact_export(),
            ContactAction::Import { nickname, path } => contact_import(&nickname, &path),
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

async fn service_join_listen() -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
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

async fn service_join(connection_string: String) -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
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

async fn service_replicate_listen() -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    ReplicaService::listen(&vault).await?;
    Ok(())
}

async fn service_replicate(to_device_name: String) -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    ReplicaService::push(&vault, &to_device_name).await?;
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
    ShareService::listen(&vault).await?;
    Ok(())
}

async fn service_share(to_nickname: String) -> anyhow::Result<()> {
    let vault = Vault::new(&std::env::current_dir()?)?;
    ShareService::share_with(&vault, &to_nickname).await?;
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

fn note_update(path: &Path, content: &str) -> anyhow::Result<()> {
    let note_path = std::env::current_dir()?.join(path);
    let mut n = Note::from_path(note_path)?;
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

use anyhow::Result;
use iroh::{Endpoint, SecretKey};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const FOOTNOTES_DIR: &str = ".footnotes";
const CONTACTS_DIR: &str = "contacts";
const TRUSTED_SOURCES_DIR: &str = "footnotes";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";
const MASTER_KEY_FILE: &str = "master_identity";
const CONTACT_FILE: &str = "contact.json";

pub const ALPN_FOOTNOTE_FILES: &[u8] = b"footnote/files";

pub struct Vault {
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum ListenEvent {
    Started { endpoint_id: String },
    Received { from: String },
    Stopped,
    Error(String),
}

impl Vault {
    pub fn is_valid(path: &Path) -> bool {
        path.join(FOOTNOTES_DIR).exists()
    }

    pub fn create(path: PathBuf, username: &str, device_name: &str) -> Result<Self> {
        if Self::is_valid(&path) {
            anyhow::bail!(
                "Vault already exists at {}. Remove it first if you want to reinitialize.",
                path.display()
            );
        }

        eprintln!("Initializing vault at {}", path.display());

        let footnotes_dir = path.join(FOOTNOTES_DIR);
        let contacts_dir = footnotes_dir.join(CONTACTS_DIR);
        let trusted_sources_dir = path.join(TRUSTED_SOURCES_DIR);

        fs::create_dir_all(&footnotes_dir)?;
        fs::create_dir_all(&contacts_dir)?;
        fs::create_dir_all(&trusted_sources_dir)?;

        eprintln!("Generating master identity key...");
        let (signing_key, verifying_key) = crypto::generate_identity_keypair();

        let master_key_file = footnotes_dir.join(MASTER_KEY_FILE);
        fs::write(&master_key_file, crypto::signing_key_to_hex(&signing_key))?;
        eprintln!(
            "Master identity key stored at {}",
            master_key_file.display()
        );

        // Generate Iroh endpoint for this device
        eprintln!("Generating Iroh endpoint for this device...");
        let secret_key = SecretKey::generate(&mut rand::rng());
        let public_key = secret_key.public();

        // Store Iroh secret key
        let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
        fs::write(&key_file, secret_key.to_bytes())?;
        eprintln!("Device Iroh key stored at {}", key_file.display());

        // Create device-specific home note at vault root
        let home_uuid = Uuid::new_v4();
        let home_filename = format!("home-{}.md", device_name);
        let home_file = path.join(&home_filename);
        let home_content = format!(
            r#"---
uuid: {}
share_with: []
---

# Home ({})

Welcome to footnote! This is your home note.
"#,
            home_uuid, device_name
        );
        fs::write(&home_file, home_content)?;
        eprintln!("Home note created at {}", home_file.display());

        // Create contact.json with initial device
        eprintln!("Creating contact record...");
        let contact_timestamp = chrono::Utc::now().to_rfc3339();

        let contact_device = crypto::ContactDevice {
            device_name: device_name.to_string(),
            iroh_endpoint_id: public_key.to_string(),
            added_at: contact_timestamp.clone(),
        };

        let mut contact_record = crypto::ContactRecord {
            username: username.to_string(),
            nickname: String::new(),
            master_public_key: crypto::verifying_key_to_hex(&verifying_key),
            devices: vec![contact_device],
            updated_at: contact_timestamp,
            signature: String::new(),
        };

        let signature = crypto::sign_contact_record(&contact_record, &signing_key)?;
        contact_record.signature = signature;

        let contact_path = footnotes_dir.join("contact.json");
        fs::write(
            &contact_path,
            serde_json::to_string_pretty(&contact_record)?,
        )?;
        eprintln!("Contact record created at {}", contact_path.display());

        eprintln!("\nVault initialization complete!");

        Ok(Self { path })
    }

    /// Open an existing vault at the given path
    pub fn open(path: PathBuf) -> Result<Self> {
        if !Self::is_valid(&path) {
            anyhow::bail!(
                "Not a valid vault: {} (missing .footnotes directory)",
                path.display()
            );
        }

        Ok(Self { path })
    }

    /// Get the vault root path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the .footnotes directory path
    pub fn footnotes_dir(&self) -> PathBuf {
        self.path.join(FOOTNOTES_DIR)
    }

    /// Get the local device name for this vault
    pub fn local_device_name(&self) -> Result<String> {
        device::get_local_device_name(&self.path)
    }

    /// Get the master public key for this vault
    pub fn master_public_key(&self) -> Result<String> {
        let contact_path = self.footnotes_dir().join("contact.json");
        let contact_content = fs::read_to_string(&contact_path)?;
        let contact_record: crypto::ContactRecord = serde_json::from_str(&contact_content)?;
        Ok(contact_record.master_public_key)
    }

    /// Get the local device's endpoint ID
    pub fn device_endpoint_id(&self) -> Result<String> {
        let key_file = self.footnotes_dir().join(LOCAL_DEVICE_KEY_FILE);
        let key_bytes = fs::read(&key_file)?;
        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
        let secret_key = SecretKey::from_bytes(&key_array);
        Ok(secret_key.public().to_string())
    }

    /// Start listening for incoming sync/share connections in the background
    /// Returns a receiver for status events and a cancellation token to stop
    pub async fn listen_background(&self) -> Result<(Receiver<ListenEvent>, CancellationToken)> {
        let (tx, rx) = mpsc::channel(32);
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();

        // Load device secret key
        let key_file = self.footnotes_dir().join(LOCAL_DEVICE_KEY_FILE);
        let key_bytes = fs::read(&key_file)?;
        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
        let secret_key = SecretKey::from_bytes(&key_array);
        let endpoint_id = secret_key.public();

        // Clone vault path for spawned task
        let vault_path = self.path.clone();
        let notes_dir = self.path.clone();

        tokio::spawn(async move {
            // Create Iroh endpoint
            let endpoint_result = Endpoint::builder()
                .secret_key(secret_key)
                .alpns(vec![sync::ALPN_FOOTNOTE_FILES.to_vec()])
                .bind()
                .await;

            let endpoint = match endpoint_result {
                Ok(ep) => ep,
                Err(e) => {
                    let _ = tx.send(ListenEvent::Error(e.to_string())).await;
                    return;
                }
            };

            // Notify started
            let _ = tx
                .send(ListenEvent::Started {
                    endpoint_id: endpoint_id.to_string(),
                })
                .await;

            // Accept connections loop
            loop {
                tokio::select! {
                    Some(incoming) = endpoint.accept() => {
                        let mut accepting = match incoming.accept() {
                            Ok(a) => a,
                            Err(e) => {
                                let _ = tx.send(ListenEvent::Error(format!("Accept error: {}", e))).await;
                                continue;
                            }
                        };

                        let alpn = match accepting.alpn().await {
                            Ok(a) => a,
                            Err(e) => {
                                let _ = tx.send(ListenEvent::Error(format!("ALPN error: {}", e))).await;
                                continue;
                            }
                        };

                        let conn = match accepting.await {
                            Ok(c) => c,
                            Err(e) => {
                                let _ = tx.send(ListenEvent::Error(format!("Connection error: {}", e))).await;
                                continue;
                            }
                        };

                        if alpn == sync::ALPN_FOOTNOTE_FILES {
                            let remote_id = conn.remote_id();

                            // Identify device (could fail, but we still handle the connection)
                            let device_name = match sync::identify_device(&vault_path, &remote_id).await {
                                Ok((_, name)) => name,
                                Err(_) => remote_id.to_string(),
                            };

                            let _ = tx.send(ListenEvent::Received { from: device_name.clone() }).await;

                            // Spawn task to handle connection
                            let notes_dir_clone = notes_dir.clone();
                            let vault_path_for_task = vault_path.clone();
                            tokio::spawn(async move {
                                if let Err(e) = sync::handle_sync_accept(&vault_path_for_task, conn, &notes_dir_clone).await {
                                    eprintln!("Error handling sync: {:?}", e);
                                }
                            });
                        }
                    }
                    _ = cancel_clone.cancelled() => {
                        // Graceful shutdown
                        let _ = tx.send(ListenEvent::Stopped).await;
                        break;
                    }
                }
            }
        });

        Ok((rx, cancel_token))
    }
}

pub async fn listen(vault_path: &std::path::Path) -> Result<()> {
    // Load this device's Iroh secret key
    let footnotes_dir = vault_path.join(".footnotes");
    let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);
    let endpoint_id = secret_key.public();

    // Get notes directory
    let notes_dir = vault_path.to_path_buf();

    println!("\n📡 Mirror Sync - Listening");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Endpoint ID: {}", endpoint_id);
    println!("Ready to receive syncs from your other devices");
    println!("\nPress Ctrl+C to stop listening");
    println!();

    // Create Iroh endpoint with protocol handler
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![sync::ALPN_FOOTNOTE_FILES.to_vec()])
        .bind()
        .await?;

    // Accept connections in a loop
    loop {
        tokio::select! {
            Some(incoming) = endpoint.accept() => {
                let mut accepting = incoming.accept()?;
                let alpn = accepting.alpn().await?;
                let conn = accepting.await?;

                if alpn == sync::ALPN_FOOTNOTE_FILES {
                    // Spawn a task to handle the connection
                    let notes_dir_clone = notes_dir.clone();
                    let vault_path_clone = vault_path.to_path_buf();
                    tokio::spawn(async move {
                        if let Err(e) = sync::handle_sync_accept(&vault_path_clone, conn, &notes_dir_clone).await {
                            eprintln!("Error handling sync: {:?}", e);
                        }
                    });
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down...");
                break;
            }
        }
    }

    Ok(())
}

// if we open files through the vault interface, the vault can save the last
// opened file. but.. maybe that's ui. could go either way.
fn load_last_session(mut vault_ctx: VaultContext, mut current_file: Signal<Option<FootnoteFile>>) {
    spawn(async move {
        if let Some(config) = crate::ui::config::AppConfig::load() {
            if !config.validate_vault() {
                tracing::info!("Config vault path invalid, ignoring config");
                return;
            }
            vault_ctx.set_vault(config.last_vault_path.clone());

            if let Some(filename) = config.last_file {
                let file_path = config.last_vault_path.join(&filename);
                if file_path.exists() {
                    if let Ok(note) = crate::core::note::parse_note(&file_path) {
                        current_file.set(Some(FootnoteFile {
                            path: file_path,
                            filename: filename.clone(),
                            content: note.content,
                            share_with: note.frontmatter.share_with,
                            footnotes: note.frontmatter.footnotes,
                        }));
                    }
                }
            }
        }
    });
}

fn load_device_home_file(vault_path: PathBuf, mut current_file: Signal<Option<FootnoteFile>>) {
    let vault_path_for_spawn = vault_path.clone();
    spawn(async move {
        let device_name = match crate::core::device::get_local_device_name(&vault_path_for_spawn) {
            Ok(name) => name,
            Err(_) => return,
        };

        let home_filename = format!("home-{}.md", device_name);
        let home_path = vault_path_for_spawn.join(&home_filename);

        if let Ok(note) = crate::core::note::parse_note(&home_path) {
            current_file.set(Some(FootnoteFile {
                path: home_path,
                filename: home_filename,
                content: note.content,
                share_with: note.frontmatter.share_with,
                footnotes: note.frontmatter.footnotes,
            }));
        }
    });
}

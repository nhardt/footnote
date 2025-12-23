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

fn contact_read() {
    // Load trusted contacts
    let contacts_dir = vault_path.join("contacts");
    if let Ok(entries) = std::fs::read_dir(contacts_dir) {
        let mut contacts = Vec::new();
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.ends_with(".json") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(contact) =
                            serde_json::from_str::<crate::core::crypto::ContactRecord>(&content)
                        {
                            let petname = file_name.trim_end_matches(".json").to_string();
                            contacts.push((petname, contact));
                        }
                    }
                }
            }
        }
        contacts.sort_by(|a, b| a.0.cmp(&b.0));
        trusted_contacts.set(contacts);
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

/// Create a new device (primary side) - generates join URL and listens for connection
pub async fn vault_device_create(
    vault_path: &std::path::Path,
) -> anyhow::Result<Receiver<DeviceAuthEvent>> {
    let (tx, rx) = mpsc::channel(32);
    // Check if this device is primary
    if !is_primary_device(vault_path)? {
        anyhow::bail!(
            "This device is not marked as primary. Only the primary device can create join URLs.\n\
            Run this command on your primary device."
        );
    }

    // Load master identity key
    let footnotes_dir = vault_path.join(".footnotes");
    let master_key_file = footnotes_dir.join(MASTER_KEY_FILE);
    if !master_key_file.exists() {
        anyhow::bail!("Master identity key not found. Run 'footnote init' first.");
    }

    let master_key_hex = fs::read_to_string(&master_key_file)?;
    let signing_key = crypto::signing_key_from_hex(&master_key_hex)?;

    // Generate one-time token
    let token = Uuid::new_v4().to_string();

    // Load this device's Iroh secret key to create endpoint
    let this_device_key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&this_device_key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);

    // Create Iroh endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key.clone())
        .alpns(vec![ALPN_DEVICE_AUTH.to_vec()])
        .bind()
        .await?;

    let endpoint_id = secret_key.public();
    let join_url = format!("iroh://{}?token={}", endpoint_id, token);

    // Send initial listening event with URL
    let _ = tx
        .send(DeviceAuthEvent::Listening {
            join_url: join_url.clone(),
        })
        .await;

    // Clone vault_path for use in spawned task
    let vault_path = vault_path.to_path_buf();

    // Spawn background task to handle connection
    tokio::spawn(async move {
        // Wait for connection
        if let Some(incoming) = endpoint.accept().await {
            let _ = tx.send(DeviceAuthEvent::Connecting).await;

            match async {
                let conn = incoming.accept()?.await?;
                let (mut send, mut recv) = conn.accept_bi().await.anyerr()?;

                // Read join request
                let request_bytes = recv.read_to_end(10000).await.anyerr()?;
                let request: DeviceJoinRequest = serde_json::from_slice(&request_bytes)?;

                let _ = tx
                    .send(DeviceAuthEvent::ReceivedRequest {
                        device_name: request.device_name.clone(),
                    })
                    .await;

                // Verify token
                if request.token != token {
                    anyhow::bail!("Invalid token. Authorization failed.");
                }

                let _ = tx.send(DeviceAuthEvent::Verifying).await;

                // Load current contact.json
                let contact_path = vault_path.join(".footnotes").join("contact.json");
                let contact_content = fs::read_to_string(&contact_path)?;
                let mut contact_record: crypto::ContactRecord =
                    serde_json::from_str(&contact_content)?;

                // Add new device to contact record
                let new_device = crypto::ContactDevice {
                    device_name: request.device_name.clone(),
                    iroh_endpoint_id: request.iroh_endpoint_id,
                    added_at: chrono::Utc::now().to_rfc3339(),
                };

                contact_record.devices.push(new_device);
                contact_record.updated_at = chrono::Utc::now().to_rfc3339();
                contact_record.signature = String::new();

                // Re-sign entire contact record
                let signature = crypto::sign_contact_record(&contact_record, &signing_key)?;
                contact_record.signature = signature;

                // Save updated contact.json locally
                fs::write(
                    &contact_path,
                    serde_json::to_string_pretty(&contact_record)?,
                )?;

                // Send complete contact record to remote device
                let response = DeviceJoinResponse {
                    contact_json: serde_json::to_string(&contact_record)?,
                };

                let response_bytes = serde_json::to_vec(&response)?;
                send.write_all(&response_bytes).await.anyerr()?;
                send.finish().anyerr()?;

                conn.closed().await;

                Ok::<_, anyhow::Error>(request.device_name.clone())
            }
            .await
            {
                Ok(device_name) => {
                    let _ = tx.send(DeviceAuthEvent::Success { device_name }).await;
                }
                Err(e) => {
                    let _ = tx.send(DeviceAuthEvent::Error(e.to_string())).await;
                }
            }
        }
    });

    Ok(rx)
}
// Create a new device (remote side) - joins using connection URL from primary
pub async fn create_remote(
    vault_path: &std::path::Path,
    connection_string: &str,
    device_name: &str,
) -> anyhow::Result<()> {
    // Check if vault already exists at the specified path
    let footnotes_check = vault_path.join(".footnotes");
    if footnotes_check.exists() {
        anyhow::bail!(
            "Vault already exists at {}. Remove it first if you want to join as a new device.",
            vault_path.display()
        );
    }

    // Parse connection string: iroh://endpoint-id?token=xyz
    let (endpoint_id, token) = parse_connection_string(connection_string)?;

    println!("\nDevice Join");
    println!("Connecting to primary device...");

    // Generate Iroh endpoint for this device
    let secret_key = SecretKey::generate(&mut rand::rng());
    let public_key = secret_key.public();

    // Create endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key.clone())
        .bind()
        .await?;

    // Connect to primary device
    let conn = endpoint.connect(endpoint_id, ALPN_DEVICE_AUTH).await?;
    let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

    println!("Connected");

    // Send join request
    let request = DeviceJoinRequest {
        device_name: device_name.to_string(),
        iroh_endpoint_id: public_key.to_string(),
        token,
    };

    let request_bytes = serde_json::to_vec(&request)?;
    send.write_all(&request_bytes).await.anyerr()?;
    send.finish().anyerr()?;

    println!("Authenticating...");

    // Receive response
    let response_bytes = recv.read_to_end(100000).await.anyerr()?;
    let response: DeviceJoinResponse = serde_json::from_slice(&response_bytes)?;

    println!("Received contact record");

    // Parse and verify contact record
    let contact_record: crypto::ContactRecord = serde_json::from_str(&response.contact_json)?;

    if !crypto::verify_contact_signature(&contact_record)? {
        anyhow::bail!("Contact signature verification failed");
    }

    println!("Contact signature verified");

    // Create vault directory structure in current directory
    let footnotes_dir = vault_path.join(".footnotes");
    let contacts_dir = footnotes_dir.join("contacts");
    let trusted_sources_dir = vault_path.join("footnotes");

    fs::create_dir_all(&footnotes_dir)?;
    fs::create_dir_all(&contacts_dir)?;
    fs::create_dir_all(&trusted_sources_dir)?;

    // Store Iroh secret key
    let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    fs::write(&key_file, secret_key.to_bytes())?;

    // Store contact.json
    let contact_path = footnotes_dir.join("contact.json");
    fs::write(
        &contact_path,
        serde_json::to_string_pretty(&contact_record)?,
    )?;

    // Create device-specific home note at vault root
    let home_uuid = Uuid::new_v4();
    let home_filename = format!("home-{}.md", device_name);
    let home_file = vault_path.join(&home_filename);
    let home_content = format!(
        r#"---
uuid: {}
share_with: []
---

# Home ({})

Welcome to footnote on {}!
"#,
        home_uuid, device_name, device_name
    );
    fs::write(&home_file, home_content)?;

    println!("\nJoin complete!");
    println!("Identity: {}", contact_record.nickname);
    println!("Master key: {}", contact_record.master_public_key);
    println!("Device: {}", device_name);
    println!("Devices in contact: {}", contact_record.devices.len());
    println!("Vault created at: {}", vault_path.display());

    conn.close(0u8.into(), b"done");
    conn.closed().await;

    Ok(())
}

/// Handle an incoming sync connection
pub async fn handle_sync_accept(
    vault_path: &Path,
    connection: Connection,
    local_notes_dir: &Path,
) -> Result<()> {
    let remote_endpoint_id = connection.remote_id();

    // Identify the remote device (either same user or trusted user)
    let (device_type, identifier) = identify_device(vault_path, &remote_endpoint_id).await?;

    // Determine target directory based on device type
    let target_dir = if device_type == "me" {
        // Mirror sync from my own device -> notes/
        println!(
            "Receiving mirror sync from {} ({})",
            identifier, remote_endpoint_id
        );
        local_notes_dir.to_path_buf()
    } else {
        // Share from trusted user -> footnotes/{petname}/
        println!(
            "Receiving shared documents from {} ({})",
            identifier, remote_endpoint_id
        );
        let footnotes_dir = vault_path.join("footnotes").join(&identifier);
        fs::create_dir_all(&footnotes_dir)?;
        footnotes_dir
    };

    // Open bidirectional stream
    let (mut send, mut recv) = connection.accept_bi().await.anyerr()?;

    // Read manifest length (4 bytes, u32 big-endian)
    let mut len_buf = [0u8; 4];
    RecvStream::read_exact(&mut recv, &mut len_buf)
        .await
        .anyerr()?;
    let manifest_len = u32::from_be_bytes(len_buf) as usize;

    // Read and deserialize manifest
    let mut manifest_buf = vec![0u8; manifest_len];
    RecvStream::read_exact(&mut recv, &mut manifest_buf)
        .await
        .anyerr()?;
    let remote_manifest: manifest::Manifest =
        serde_json::from_slice(&manifest_buf).context("Failed to deserialize manifest")?;

    println!("Received manifest with {} files", remote_manifest.len());

    // Create local manifest
    let local_manifest =
        manifest::create_manifest(&target_dir).context("Failed to create local manifest")?;

    // Diff: find files that need to be synced
    let files_to_sync = manifest::diff_manifests(&local_manifest, &remote_manifest);

    println!("Requesting {} files", files_to_sync.len());

    // Request and receive each file
    for file_to_sync in &files_to_sync {
        // Send file request: path length (4 bytes) + path
        let path_str = file_to_sync
            .path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        let path_bytes = path_str.as_bytes();
        let path_len = path_bytes.len() as u32;

        SendStream::write_all(&mut send, &path_len.to_be_bytes())
            .await
            .anyerr()?;
        SendStream::write_all(&mut send, path_bytes)
            .await
            .anyerr()?;

        // Receive file length (8 bytes, u64 big-endian)
        let mut file_len_buf = [0u8; 8];
        RecvStream::read_exact(&mut recv, &mut file_len_buf)
            .await
            .anyerr()?;
        let file_len = u64::from_be_bytes(file_len_buf) as usize;

        // Receive file contents
        let mut file_contents = vec![0u8; file_len];
        RecvStream::read_exact(&mut recv, &mut file_contents)
            .await
            .anyerr()?;

        // Write file to disk
        let full_path = target_dir.join(&file_to_sync.path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, file_contents)?;

        println!("  Synced: {} ({})", path_str, file_to_sync.reason_str());
    }

    // Send EOF signal (0-length path)
    SendStream::write_all(&mut send, &0u32.to_be_bytes())
        .await
        .anyerr()?;
    SendStream::finish(&mut send).anyerr()?;

    // Note: We do NOT delete local files (additive only, per design)

    println!("Sync complete! Received {} files", files_to_sync.len());
    connection.closed().await;

    Ok(())
}

/// Push files to a remote device
pub async fn push_to_device(
    local_notes_dir: &Path,
    remote_endpoint_id: iroh::PublicKey,
    local_secret_key: iroh::SecretKey,
) -> Result<()> {
    // Create manifest of local notes
    let manifest =
        manifest::create_manifest(local_notes_dir).context("Failed to create manifest")?;

    println!("Pushing {} files", manifest.len());

    // Create endpoint and connect
    let endpoint = iroh::Endpoint::builder()
        .secret_key(local_secret_key)
        .bind()
        .await?;

    let conn = endpoint
        .connect(remote_endpoint_id, ALPN_FOOTNOTE_FILES)
        .await
        .context("Failed to connect to remote device")?;

    let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

    // Serialize and send manifest
    let encoded = serde_json::to_vec(&manifest).context("Failed to serialize manifest")?;
    let len = encoded.len() as u32;
    SendStream::write_all(&mut send, &len.to_be_bytes())
        .await
        .anyerr()?;
    SendStream::write_all(&mut send, &encoded).await.anyerr()?;

    println!("Manifest sent, waiting for file requests...");

    // Loop: read file requests and serve files
    let mut files_sent = 0;
    loop {
        // Read file path length
        let mut path_len_buf = [0u8; 4];
        RecvStream::read_exact(&mut recv, &mut path_len_buf)
            .await
            .anyerr()?;
        let path_len = u32::from_be_bytes(path_len_buf);

        // EOF signal (0-length path)
        if path_len == 0 {
            println!("Received EOF signal");
            break;
        }

        // Read file path
        let mut path_buf = vec![0u8; path_len as usize];
        RecvStream::read_exact(&mut recv, &mut path_buf)
            .await
            .anyerr()?;
        let file_path = String::from_utf8(path_buf).context("Invalid UTF-8 in file path")?;

        let path = PathBuf::from(&file_path);

        // Verify file is in manifest (by looking up UUID)
        let _entry = manifest
            .entries
            .values()
            .find(|e| e.path == path)
            .ok_or_else(|| anyhow::anyhow!("Requested file not in manifest: {}", file_path))?;

        // Read file from disk
        let full_path = local_notes_dir.join(&path);
        let file_contents = fs::read(&full_path)
            .with_context(|| format!("Failed to read file: {}", full_path.display()))?;

        // Send file length + contents
        let file_len = file_contents.len() as u64;
        SendStream::write_all(&mut send, &file_len.to_be_bytes())
            .await
            .anyerr()?;
        SendStream::write_all(&mut send, &file_contents)
            .await
            .anyerr()?;

        files_sent += 1;
        println!("✓ Sent: {} ({} bytes)", file_path, file_len);
    }

    SendStream::finish(&mut send).anyerr()?;
    conn.close(0u8.into(), b"done");
    conn.closed().await;

    println!("Push complete! Sent {} files", files_sent);

    Ok(())
}

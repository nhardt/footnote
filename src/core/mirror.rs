use anyhow::Result;
use iroh::{Endpoint, SecretKey};
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

use super::{crypto, note, sync};

const LOCAL_DEVICE_KEY_FILE: &str = "this_device";

/// Events emitted during sync listening
#[derive(Debug, Clone)]
pub enum ListenEvent {
    /// Listener started successfully
    Started { endpoint_id: String },
    /// Received sync from a device
    Received { from: String },
    /// Listener stopped
    Stopped,
    /// Error occurred
    Error(String),
}

/// Start listening for incoming sync connections in the background
/// Returns a receiver for status events and a cancellation token to stop
pub async fn listen_background(vault_path: &std::path::Path) -> Result<(Receiver<ListenEvent>, CancellationToken)> {
    let (tx, rx) = mpsc::channel(32);
    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    // Load device secret key
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
    let vault_path_clone = vault_path.to_path_buf();

    tokio::spawn(async move {
        // Create Iroh endpoint
        let endpoint_result = Endpoint::builder()
            .secret_key(secret_key)
            .alpns(vec![sync::ALPN_MIRROR.to_vec()])
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
        let _ = tx.send(ListenEvent::Started {
            endpoint_id: endpoint_id.to_string(),
        }).await;

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

                    if alpn == sync::ALPN_MIRROR {
                        let remote_id = conn.remote_id();

                        // Identify device (could fail, but we still handle the connection)
                        let device_name = match crate::core::sync::identify_device(&vault_path_clone, &remote_id).await {
                            Ok((_, name)) => name,
                            Err(_) => remote_id.to_string(),
                        };

                        let _ = tx.send(ListenEvent::Received { from: device_name.clone() }).await;

                        // Spawn task to handle connection
                        let notes_dir_clone = notes_dir.clone();
                        let vault_path_for_task = vault_path_clone.clone();
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

/// Listen for incoming mirror connections
///
/// Starts an Iroh endpoint listening for sync connections from other devices.
/// Only accepts connections from devices belonging to the same user (verified via signatures).
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
        .alpns(vec![sync::ALPN_MIRROR.to_vec()])
        .bind()
        .await?;

    // Accept connections in a loop
    loop {
        tokio::select! {
            Some(incoming) = endpoint.accept() => {
                let mut accepting = incoming.accept()?;
                let alpn = accepting.alpn().await?;
                let conn = accepting.await?;

                if alpn == sync::ALPN_MIRROR {
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

/// Push mirror data to another device
///
/// For self-to-self sync (initial implementation):
/// - Must specify --device parameter
/// - Pushes all notes to the specified device (same user)
///
/// Future: --user parameter for user-to-user sharing
pub async fn push(vault_path: &std::path::Path, user: Option<&str>, device: Option<&str>) -> Result<()> {
    match (user, device) {
        (None, Some(device_name)) => {
            // Self-to-self sync: push to specified device
            push_to_own_device(vault_path, device_name).await
        }
        (None, None) => {
            anyhow::bail!(
                "Please specify a device to push to using --device\n\
                 Example: footnote mirror push --device laptop"
            );
        }
        (Some(_user_name), _) => {
            anyhow::bail!(
                "User-to-user sharing is not yet implemented.\n\
                 For now, only self-to-self sync is supported.\n\
                 Use: footnote mirror push --device <device_name>"
            );
        }
    }
}

/// Push to one of the user's own devices (self-to-self sync)
async fn push_to_own_device(vault_path: &std::path::Path, device_name: &str) -> Result<()> {
    // Load this device's Iroh secret key
    let footnotes_dir = vault_path.join(".footnotes");
    let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);

    // Look up the target device in contact.json
    let contact_path = footnotes_dir.join("contact.json");
    let contact_content = fs::read_to_string(&contact_path)?;
    let contact_record: crate::core::crypto::ContactRecord =
        serde_json::from_str(&contact_content)?;

    let device = contact_record
        .devices
        .iter()
        .find(|d| d.device_name == device_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Device '{}' not found.\n\
                 Available devices can be seen with: footnote user read",
                device_name
            )
        })?;

    let endpoint_id = device.iroh_endpoint_id.parse::<iroh::PublicKey>()?;

    println!("\n📤 Mirror Sync - Push");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Target device: {}", device_name);
    println!("Target endpoint: {}", endpoint_id);
    println!();

    // Get notes directory
    let notes_dir = vault_path.to_path_buf();

    // Push to the device
    sync::push_to_device(&notes_dir, endpoint_id, secret_key).await?;

    Ok(())
}

/// Share documents with trusted users based on frontmatter share_with field
pub async fn share(vault_path: &std::path::Path, petname_filter: Option<&str>) -> Result<()> {
    // Load this device's Iroh secret key
    let footnotes_dir = vault_path.join(".footnotes");
    let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);

    // (Future: could use my contact record to determine my petname from their perspective)

    // Collect all notes and group by share_with users
    let notes_dir = vault_path.to_path_buf();
    let mut shared_docs: std::collections::HashMap<String, Vec<PathBuf>> = std::collections::HashMap::new();

    println!("\nDocument Sharing");
    println!("============================================");

    // Scan all notes
    for entry in WalkDir::new(&notes_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            // Parse frontmatter
            if let Ok(frontmatter) = note::get_frontmatter(entry.path()) {
                // Check if this document is shared with anyone
                for petname in &frontmatter.share_with {
                    // If filter is specified, only include that user
                    if let Some(filter) = petname_filter {
                        if petname != filter {
                            continue;
                        }
                    }

                    shared_docs.entry(petname.clone())
                        .or_insert_with(Vec::new)
                        .push(entry.path().to_path_buf());
                }
            }
        }
    }

    if shared_docs.is_empty() {
        println!("No documents marked for sharing");
        if let Some(filter) = petname_filter {
            println!("(No documents with share_with: [{}])", filter);
        }
        return Ok(());
    }

    println!("Found {} user(s) to share with:\n", shared_docs.len());

    // Share with each user
    for (petname, docs) in shared_docs {
        println!("Sharing with '{}':", petname);
        println!("  {} document(s)", docs.len());

        // Look up the user's contact record
        let contact_file_path = footnotes_dir.join("contacts").join(format!("{}.json", petname));
        if !contact_file_path.exists() {
            eprintln!("  Warning: Contact not found for '{}'. Run 'footnote trust' first.", petname);
            eprintln!("  Skipping...\n");
            continue;
        }

        let contact_content = fs::read_to_string(&contact_file_path)?;
        let user_contact: crypto::ContactRecord = serde_json::from_str(&contact_content)?;

        // Find their primary device
        let primary_device = user_contact.devices.iter()
            .find(|d| d.device_name == user_contact.username || d.device_name == "desktop" || d.device_name == "primary")
            .or_else(|| user_contact.devices.first())
            .ok_or_else(|| anyhow::anyhow!("User '{}' has no devices", petname))?;

        let endpoint_id = primary_device.iroh_endpoint_id.parse::<iroh::PublicKey>()?;

        // Create a temporary directory with the documents to share
        let temp_dir = std::env::temp_dir().join(format!("footnote-share-{}", petname));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        // Copy the shared documents to temp directory
        for doc_path in &docs {
            let file_name = doc_path.file_name().unwrap();
            let dest_path = temp_dir.join(file_name);
            fs::copy(doc_path, dest_path)?;
        }

        // Connect and share
        println!("  Connecting to {}...", endpoint_id);

        // Push the documents using sync protocol
        match sync::push_to_device(&temp_dir, endpoint_id, secret_key.clone()).await {
            Ok(_) => println!("  [OK] Shared successfully\n"),
            Err(e) => {
                eprintln!("  [FAIL] Failed to share: {}\n", e);
            }
        }

        // Cleanup temp directory
        fs::remove_dir_all(&temp_dir)?;
    }

    println!("============================================");
    println!("Sharing complete");

    Ok(())
}

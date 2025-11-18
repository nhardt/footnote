use crate::core::{crypto, vault};
use iroh::{Endpoint, SecretKey};
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

const ALPN_DEVICE_AUTH: &[u8] = b"fieldnote/device-auth";
const MASTER_KEY_FILE: &str = "master_identity";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";

#[derive(Debug, Serialize, Deserialize)]
struct DeviceJoinRequest {
    device_name: String,
    iroh_endpoint_id: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceJoinResponse {
    master_public_key: String,
    nickname: String,
    device_name: String,
    iroh_endpoint_id: String,
    authorized_by: String,
    timestamp: String,
    signature: String,
}

/// Check if the current device is a primary device
fn is_primary_device() -> anyhow::Result<bool> {
    let keys_dir = vault::get_keys_dir()?;
    let master_key_file = keys_dir.join(MASTER_KEY_FILE);
    Ok(master_key_file.exists())
}

/// Get the local device name by matching the public key
pub fn get_local_device_name() -> anyhow::Result<String> {
    let keys_dir = vault::get_keys_dir()?;
    let key_file = keys_dir.join(LOCAL_DEVICE_KEY_FILE);

    if !key_file.exists() {
        anyhow::bail!(
            "Local device key not found at {}. Run 'fieldnote init' first.",
            key_file.display()
        );
    }

    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);
    let local_public_key = secret_key.public();

    let devices_dir = vault::get_devices_dir()?;

    for entry in fs::read_dir(&devices_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        if let Ok(endpoint_id) = parse_device_endpoint_id(&content) {
            if endpoint_id == local_public_key {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    return Ok(name.to_string());
                }
            }
        }
    }

    anyhow::bail!(
        "Could not find device file matching local public key {}. \
         Your vault may be corrupted.",
        local_public_key
    )
}

fn parse_device_endpoint_id(content: &str) -> anyhow::Result<iroh::PublicKey> {
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        anyhow::bail!("Invalid device file format");
    }

    let rest = if content.starts_with("---\r\n") {
        &content[5..]
    } else {
        &content[4..]
    };

    let end_pos = rest
        .find("\n---\n")
        .or_else(|| rest.find("\r\n---\r\n"))
        .ok_or_else(|| anyhow::anyhow!("Frontmatter not properly closed"))?;

    let frontmatter = &rest[..end_pos];

    for line in frontmatter.lines() {
        if let Some(stripped) = line.trim().strip_prefix("iroh_endpoint_id:") {
            let endpoint_str = stripped.trim().trim_matches('"');
            return endpoint_str
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid endpoint ID"));
        }
    }

    anyhow::bail!("iroh_endpoint_id not found in device file")
}

/// Delete a device
pub async fn delete(user_name: &str, device_name: &str) -> anyhow::Result<()> {
    println!(
        "TODO: device::delete({}, {})",
        user_name, device_name
    );
    Ok(())
}

/// Create a new device (primary side) - generates join URL and listens for connection
pub async fn create_primary() -> anyhow::Result<()> {
    // Check if this device is primary
    if !is_primary_device()? {
        anyhow::bail!(
            "This device is not marked as primary. Only the primary device can create join URLs.\n\
            Run this command on your primary device."
        );
    }

    // Load master identity key
    let keys_dir = vault::get_keys_dir()?;
    let master_key_file = keys_dir.join(MASTER_KEY_FILE);
    if !master_key_file.exists() {
        anyhow::bail!("Master identity key not found. Run 'fieldnote init' first.");
    }

    let master_key_hex = fs::read_to_string(&master_key_file)?;
    let signing_key = crypto::signing_key_from_hex(&master_key_hex)?;
    let verifying_key = signing_key.verifying_key();

    // Load identity to get nickname
    let identity_path = vault::get_identity_path()?;
    let identity_content = fs::read_to_string(&identity_path)?;
    let nickname = parse_nickname(&identity_content)?;

    // Generate one-time token
    let token = Uuid::new_v4().to_string();

    // Load this device's Iroh secret key to create endpoint
    let this_device_key_file = keys_dir.join(LOCAL_DEVICE_KEY_FILE);
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

    println!("\n🔐 Device Authorization");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Listening for new device...");
    println!("\nCopy this to your new device:");
    println!("  iroh://{}?token={}", endpoint_id, token);
    println!("\nWaiting for connection...");
    println!("(Press Ctrl+C to cancel)");

    // Wait for connection
    if let Some(incoming) = endpoint.accept().await {
        println!("\n✓ Device connecting...");
        let conn = incoming.accept()?.await?;
        let (mut send, mut recv) = conn.accept_bi().await.anyerr()?;

        // Read join request
        let request_bytes = recv.read_to_end(10000).await.anyerr()?;
        let request: DeviceJoinRequest = serde_json::from_slice(&request_bytes)?;

        println!("✓ Received join request from device '{}'", request.device_name);

        // Verify token
        if request.token != token {
            anyhow::bail!("Invalid token. Authorization failed.");
        }

        println!("✓ Token verified");

        // Sign the device record
        let timestamp = chrono::Utc::now().to_rfc3339();
        let signature = crypto::sign_device_record(
            &request.device_name,
            &request.iroh_endpoint_id,
            &signing_key,
            &timestamp,
        )?;

        println!("✓ Device record signed");

        // Send response
        let response = DeviceJoinResponse {
            master_public_key: crypto::verifying_key_to_hex(&verifying_key),
            nickname,
            device_name: request.device_name.clone(),
            iroh_endpoint_id: request.iroh_endpoint_id,
            authorized_by: crypto::verifying_key_to_hex(&verifying_key),
            timestamp,
            signature,
        };

        let response_bytes = serde_json::to_vec(&response)?;
        send.write_all(&response_bytes).await.anyerr()?;
        send.finish().anyerr()?;

        println!("✓ Authorization complete!");
        println!("\nDevice '{}' has been authorized", request.device_name);

        conn.closed().await;
    }

    Ok(())
}

/// Create a new device (remote side) - joins using connection URL from primary
pub async fn create_remote(connection_string: &str, device_name: &str) -> anyhow::Result<()> {
    // Check if vault already exists
    let vault_path = vault::get_vault_path()?;
    if vault_path.exists() {
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

    println!("Received identity");

    // Create vault directory structure
    let keys_dir = vault::get_keys_dir()?;
    let devices_dir = vault::get_devices_dir()?;
    let notes_dir = vault::get_notes_dir()?;
    let outpost_dir = vault::get_outpost_dir()?;

    fs::create_dir_all(&keys_dir)?;
    fs::create_dir_all(&devices_dir)?;
    fs::create_dir_all(&notes_dir)?;
    fs::create_dir_all(&outpost_dir)?;

    // Store Iroh secret key
    let key_file = keys_dir.join(LOCAL_DEVICE_KEY_FILE);
    fs::write(&key_file, secret_key.to_bytes())?;

    // Create identity.md
    let identity_file = vault::get_identity_path()?;
    let identity_content = format!(
        r#"---
master_public_key: {}
nickname: {}
---

# My Identity

This device joined an existing identity.
"#,
        response.master_public_key, response.nickname
    );
    fs::write(&identity_file, identity_content)?;

    // Create device file
    let device_file = devices_dir.join(format!("{}.md", device_name));
    let device_content = format!(
        r#"---
device_name: {}
iroh_endpoint_id: {}
authorized_by: {}
timestamp: {}
signature: {}
---

This device joined via authorization.
"#,
        response.device_name,
        response.iroh_endpoint_id,
        response.authorized_by,
        response.timestamp,
        response.signature
    );
    fs::write(&device_file, device_content)?;

    // Create home note
    let home_uuid = Uuid::new_v4();
    let home_file = notes_dir.join("home.md");
    let home_content = format!(
        r#"---
uuid: {}
share_with: []
---

# Home

Welcome to fieldnote on {}!
"#,
        home_uuid, device_name
    );
    fs::write(&home_file, home_content)?;

    println!("\nJoin complete!");
    println!("Identity: {}", response.nickname);
    println!("Master key: {}", response.master_public_key);
    println!("Device: {}", device_name);
    println!("Vault created at: {}", vault_path.display());

    conn.close(0u8.into(), b"done");
    conn.closed().await;

    Ok(())
}

fn parse_connection_string(conn_str: &str) -> anyhow::Result<(iroh::PublicKey, String)> {
    // Expected format: iroh://endpoint-id?token=xyz
    let conn_str = conn_str.trim();

    if !conn_str.starts_with("iroh://") {
        anyhow::bail!("Invalid connection string. Expected format: iroh://endpoint-id?token=xyz");
    }

    let without_prefix = conn_str.strip_prefix("iroh://").unwrap();
    let parts: Vec<&str> = without_prefix.split('?').collect();

    if parts.len() != 2 {
        anyhow::bail!("Invalid connection string. Expected format: iroh://endpoint-id?token=xyz");
    }

    let endpoint_id: iroh::PublicKey = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid endpoint ID"))?;

    // Parse token from query string
    let query = parts[1];
    let token = query
        .split('&')
        .find(|s| s.starts_with("token="))
        .and_then(|s| s.strip_prefix("token="))
        .ok_or_else(|| anyhow::anyhow!("Token not found in connection string"))?
        .to_string();

    Ok((endpoint_id, token))
}

fn parse_nickname(content: &str) -> anyhow::Result<String> {
    let mut lines = content.lines();

    // Check for opening ---
    if lines.next().map(|l| l.trim()) != Some("---") {
        return Ok(String::new());
    }

    // Parse YAML frontmatter
    let mut frontmatter_lines = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        frontmatter_lines.push(line);
    }

    let frontmatter = frontmatter_lines.join("\n");

    // Simple nickname extraction
    for line in frontmatter.lines() {
        if line.trim().starts_with("nickname:") {
            let nickname = line
                .trim()
                .strip_prefix("nickname:")
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();
            return Ok(nickname);
        }
    }

    Ok(String::new())
}

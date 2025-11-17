use crate::core::{crypto, vault};
use iroh::{Endpoint, SecretKey};
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

const ALPN_DEVICE_AUTH: &[u8] = b"fieldnote/device-auth";
const MASTER_KEY_FILE: &str = "master_identity";

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

/// Create a new device for a user
pub async fn create(user_name: &str, device_name: &str) -> anyhow::Result<()> {
    println!(
        "TODO: device::create({}, {})",
        user_name, device_name
    );
    Ok(())
}

/// Delete a device
pub async fn delete(user_name: &str, device_name: &str) -> anyhow::Result<()> {
    println!(
        "TODO: device::delete({}, {})",
        user_name, device_name
    );
    Ok(())
}

/// Listen for a new device to authorize
pub async fn authorize_listen() -> anyhow::Result<()> {
    let vault_path = vault::get_vault_path()?;
    if !vault_path.exists() {
        anyhow::bail!(
            "Vault not found at {}. Run 'fieldnote init' first.",
            vault_path.display()
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
    let this_device_key_file = keys_dir.join("this_device");
    if !this_device_key_file.exists() {
        anyhow::bail!("This device's key not found. Run 'fieldnote init' first.");
    }

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

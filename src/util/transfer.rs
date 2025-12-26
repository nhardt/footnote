use crate::model::vault::Vault;
use crate::util::manifest::{create_manifest_full, diff_manifests, Manifest};
use crate::util::network;
use anyhow::{Context, Result};
use iroh::endpoint::Connection;
use iroh::Endpoint;
use std::fs;
use std::path::PathBuf;

/// file exchange protocol:
/// - on push from device A to device B
///
/// A creates manifest for B
/// A sends manifest to B
/// B reads manifest, looks for needed files
/// B requests files from A
/// A verifies B can read requested file
/// A sends file to B

pub async fn receive_files(vault: &Vault, connection: Connection) -> Result<()> {
    let (mut send, mut recv) = connection.accept_bi().await?;

    let manifest_bytes = network::receive_bytes(&mut recv).await?;
    let remote_manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).context("Failed to deserialize manifest")?;

    let local_manifest =
        create_manifest_full(&vault.path).context("Failed to create local manifest")?;
    let files_to_sync = diff_manifests(&local_manifest, &remote_manifest);

    for file_to_sync in &files_to_sync {
        network::send_file_request(&mut send, &file_to_sync.uuid).await?;
        let file_contents = network::receive_file_contents(&mut recv).await?;
        let full_path = vault.path.join(&file_to_sync.path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, file_contents)?;
    }

    network::send_eof(&mut send).await?;
    connection.closed().await;
    Ok(())
}

pub async fn push_manifest_to_endpoint(
    vault: &Vault,
    manifest: Manifest,
    remote_endpoint_id: iroh::PublicKey,
    alpn: &[u8],
) -> Result<()> {
    let (secret_key, _) = vault.device_secret_key()?;
    let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
    let conn = endpoint
        .connect(remote_endpoint_id, alpn)
        .await
        .context("Failed to connect to remote device")?;
    let (mut send, mut recv) = conn.open_bi().await?;
    let encoded = serde_json::to_vec(&manifest).context("Failed to serialize manifest")?;

    network::send_bytes(&mut send, &encoded).await?;

    loop {
        let file_uuid = match network::receive_file_request(&mut recv).await? {
            Some(uuid) => uuid,
            None => break,
        };

        let entry = manifest
            .get(&file_uuid)
            .ok_or_else(|| anyhow::anyhow!("Requested file UUID not in manifest"))?;

        if !vault.can_device_read_note(&remote_endpoint_id, &entry.path)? {
            continue;
        }

        let full_path = vault.path.join(&entry.path);
        let file_contents = fs::read(&full_path)
            .with_context(|| format!("Failed to read file: {}", full_path.display()))?;

        network::send_file_contents(&mut send, &file_contents).await?;
    }

    conn.close(0u8.into(), b"done");
    conn.closed().await;

    Ok(())
}

use crate::model::vault::Vault;
use crate::util::manifest::{create_manifest_full, diff_manifests, Manifest};
use crate::util::network;
use crate::util::sync_status_record::{SyncDirection, SyncStatusRecord, SyncType};
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

pub async fn receive_share(vault: &Vault, nickname: &str, connection: Connection) -> Result<()> {
    // Important Note: The peer that calls open_bi must write to its SendStream
    // before the peer Connection is able to accept the stream using
    // accept_bi(). Calling open_bi then waiting on the RecvStream without
    // writing anything to the connected SendStream will never succeed.
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
        let full_path = vault
            .path
            .join("footnotes")
            .join(nickname)
            .join(&file_to_sync.path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, file_contents)?;
    }

    network::send_eof(&mut send).await?;
    connection.closed().await;
    Ok(())
}

pub async fn receive_mirror(vault: &Vault, connection: Connection) -> Result<()> {
    let Ok(mut transfer_record) = SyncStatusRecord::start(
        vault.base_path(),
        connection.remote_id(),
        SyncType::Mirror,
        SyncDirection::Inbound,
    ) else {
        anyhow::bail!("could not create log for transfer");
    };
    // Important Note: The peer that calls open_bi must write to its SendStream
    // before the peer Connection is able to accept the stream using
    // accept_bi(). Calling open_bi then waiting on the RecvStream without
    // writing anything to the connected SendStream will never succeed.
    let (mut send, mut recv) = connection.accept_bi().await?;
    let manifest_bytes = network::receive_bytes(&mut recv).await?;

    let remote_manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).context("Failed to deserialize manifest")?;
    let local_manifest =
        create_manifest_full(&vault.path).context("Failed to create local manifest")?;
    let files_to_sync = diff_manifests(&local_manifest, &remote_manifest);

    if let Err(e) = transfer_record.update(0, Some(files_to_sync.len())) {
        tracing::warn!("could not update transfer record: {}", e);
    }
    let mut transfer_count = 0;
    for file_to_sync in &files_to_sync {
        network::send_file_request(&mut send, &file_to_sync.uuid).await?;
        let file_contents = network::receive_file_contents(&mut recv).await?;
        let full_path = vault.path.join(&file_to_sync.path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, file_contents)?;
        transfer_count += 1;
        transfer_record.update(transfer_count, None)?;
    }
    transfer_record.record_success()?;
    network::send_eof(&mut send).await?;
    connection.closed().await;
    Ok(())
}

pub async fn sync_to_target(
    vault: &Vault,
    endpoint: Endpoint,
    sync_type: SyncType,
    manifest: Manifest,
    remote_endpoint_id: iroh::PublicKey,
    alpn: &[u8],
) -> Result<()> {
    let Ok(mut transfer_record) = SyncStatusRecord::start(
        vault.base_path(),
        remote_endpoint_id,
        sync_type,
        SyncDirection::Outbound,
    ) else {
        anyhow::bail!("could not create log record");
    };

    let (secret_key, _) = vault.device_secret_key()?;

    if remote_endpoint_id == secret_key.public() {
        transfer_record.record_failure("attempting to sync with self")?;
        anyhow::bail!("cannot replicate to self");
    }

    let conn = endpoint
        .connect(remote_endpoint_id, alpn)
        .await
        .context("Failed to connect to remote device")?;

    let serialised_manifest =
        serde_json::to_vec(&manifest).context("Failed to serialize manifest")?;
    // Calling open_bi then waiting on the RecvStream without writing anything
    // to SendStream will never succeed.
    let (mut send, mut recv) = conn.open_bi().await?;
    network::send_bytes(&mut send, &serialised_manifest).await?;

    let mut files_transferred = 0;
    loop {
        let file_uuid = match network::receive_file_request(&mut recv).await? {
            Some(uuid) => uuid,
            None => break,
        };

        let entry = manifest
            .get(&file_uuid)
            .ok_or_else(|| anyhow::anyhow!("Requested file UUID not in manifest"))?;

        let full_path = vault.path.join(&entry.path);
        if !vault.can_device_read_note(&remote_endpoint_id, &full_path)? {
            continue;
        }

        let file_contents = fs::read(&full_path)
            .with_context(|| format!("Failed to read file: {}", full_path.display()))?;

        network::send_file_contents(&mut send, &file_contents).await?;
        files_transferred += 1;
        if let Err(e) = transfer_record.update(files_transferred, None) {
            tracing::warn!("could not update status: {}", e);
        }
    }

    transfer_record.record_success()?;

    conn.close(0u8.into(), b"done");
    conn.closed().await;
    Ok(())
}

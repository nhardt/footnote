use crate::model::contact::Contact;
use crate::model::vault::Vault;
use crate::util::manifest::{create_manifest_full, diff_manifests, Manifest};
use crate::util::network;
use crate::util::sync_status_record::{RecentFile, SyncDirection, SyncStatusRecord, SyncType};
use anyhow::{Context, Result};
use iroh::endpoint::Connection;
use iroh::Endpoint;
use std::fs;
use std::path::Component;

pub async fn receive_share(vault: &Vault, nickname: &str, connection: Connection) -> Result<()> {
    let Ok(mut transfer_record) = SyncStatusRecord::start(
        vault.base_path(),
        connection.remote_id(),
        SyncType::Share,
        SyncDirection::Inbound,
    ) else {
        anyhow::bail!("could not create log for transfer");
    };
    // Important Note: The peer that calls open_bi must write to its SendStream
    // before the peer Connection is able to accept the stream using
    // accept_bi(). Calling open_bi then waiting on the RecvStream without
    // writing anything to the connected SendStream will never succeed.
    let (mut send, mut recv) = connection.accept_bi().await?;

    let contact_record_bytes = network::receive_bytes(&mut recv).await?;
    if contact_record_bytes.is_empty() {
        tracing::error!("Expected contact record for {}", nickname);
        anyhow::bail!("expected contact record");
    }

    let mut incoming_contact: Contact = serde_json::from_slice(&contact_record_bytes)?;
    incoming_contact.verify()?;
    vault.contact_update(nickname, &mut incoming_contact)?;

    let contacts_bytes = network::receive_bytes(&mut recv).await?;
    let incoming_contacts: Vec<Contact> =
        serde_json::from_slice(&contacts_bytes).context("Failed to deserialize contact records")?;

    if !incoming_contacts.is_empty() {
        tracing::error!("received non empty contact list from share peer");
    }

    let manifest_bytes = network::receive_bytes(&mut recv).await?;
    let remote_manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).context("Failed to deserialize manifest")?;
    let local_manifest =
        create_manifest_full(&vault.path).context("Failed to create local manifest")?;
    let files_to_sync = diff_manifests(&local_manifest, &remote_manifest);
    if let Err(e) = transfer_record.update(0, Some(files_to_sync.len())) {
        tracing::warn!("could not update transfer record: {}", e);
    }
    for file_to_sync in &files_to_sync {
        let path_components: Vec<_> = file_to_sync.path.components().collect();
        for component in &path_components {
            match component {
                Component::ParentDir => {
                    anyhow::bail!("Path traversal attempt: {:?}", file_to_sync.path);
                }
                Component::Prefix(_) | Component::RootDir => {
                    anyhow::bail!("Absolute path not allowed: {:?}", file_to_sync.path);
                }
                Component::Normal(_) | Component::CurDir => {
                    // expected
                }
            }
        }

        let full_path = vault
            .path
            .join("footnotes")
            .join(nickname)
            .join(&file_to_sync.path);
        let contact_base = vault.path.join("footnotes").join(nickname);
        let canonical_base = contact_base
            .canonicalize()
            .or_else(|_| {
                // If contact directory doesn't exist yet, create it and canonicalize
                fs::create_dir_all(&contact_base)?;
                contact_base.canonicalize()
            })
            .context("Failed to canonicalize contact base path")?;

        let canonical_full = if full_path.exists() {
            full_path.canonicalize()?
        } else {
            let parent = full_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path has no parent"))?;
            fs::create_dir_all(parent)?;
            let canonical_parent = parent.canonicalize()?;
            let filename = full_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Path has no filename"))?;
            canonical_parent.join(filename)
        };

        if !canonical_full.starts_with(&canonical_base) {
            anyhow::bail!(
                "Path escapes contact directory: {:?} (contact: {})",
                file_to_sync.path,
                nickname
            );
        }

        network::send_file_request(&mut send, &file_to_sync.uuid).await?;
        let file_contents = network::receive_file_contents(&mut recv).await?;
        let temp_path = canonical_full.with_extension("tmp");
        fs::write(&temp_path, file_contents)?;
        fs::rename(&temp_path, &canonical_full)?;

        transfer_record.record_file_complete(RecentFile {
            uuid: file_to_sync.uuid,
            filename: file_to_sync.path.to_string_lossy().to_string(),
            timestamp: file_to_sync.modified,
        });
    }
    transfer_record.record_success()?;
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
    let user_record_bytes = network::receive_bytes(&mut recv).await?;
    if user_record_bytes.is_empty() {
        anyhow::bail!("expected user record");
    }

    let incoming_user_record: Contact = serde_json::from_slice(&user_record_bytes)?;
    incoming_user_record.verify()?;

    let Some(user_record) = vault.user_read()? else {
        //TODO: we may be able to consolidate the pairing code with this code by
        // allowing a user record on our very first sync
        anyhow::bail!("cannot receive sync without user record");
    };

    if let Err(e) = incoming_user_record.is_valid_successor_of(&user_record) {
        tracing::error!("failed successor check: {}", e);
        anyhow::bail!("received invalid user record update");
    }
    vault.user_write(&incoming_user_record)?;

    let contacts_bytes = network::receive_bytes(&mut recv).await?;
    let incoming_contacts: Vec<Contact> =
        serde_json::from_slice(&contacts_bytes).context("Failed to deserialize contact records")?;

    if !incoming_contacts.is_empty() {
        let sender_is_leader = incoming_user_record
            .device_leader
            .parse::<iroh::PublicKey>()
            .map(|leader_key| leader_key == connection.remote_id())
            .unwrap_or(false);

        if sender_is_leader {
            if let Err(e) = vault.contacts_replace(&incoming_contacts) {
                tracing::error!("failed to sync contacts from mirror: {}", e);
            }
        } else {
            tracing::warn!(
                "received contacts from non-manager device {}, ignoring",
                connection.remote_id()
            );
        }
    }

    let manifest_bytes = network::receive_bytes(&mut recv).await?;

    let remote_manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).context("Failed to deserialize manifest")?;
    let local_manifest =
        create_manifest_full(&vault.path).context("Failed to create local manifest")?;
    let files_to_sync = diff_manifests(&local_manifest, &remote_manifest);

    if let Err(e) = transfer_record.update(0, Some(files_to_sync.len())) {
        tracing::warn!("could not update transfer record: {}", e);
    }
    for file_to_sync in &files_to_sync {
        let path_components: Vec<_> = file_to_sync.path.components().collect();
        for component in &path_components {
            match component {
                Component::ParentDir => {
                    anyhow::bail!("Path traversal attempt: {:?}", file_to_sync.path);
                }
                Component::Prefix(_) | Component::RootDir => {
                    anyhow::bail!("Absolute path not allowed: {:?}", file_to_sync.path);
                }
                Component::Normal(_) | Component::CurDir => {
                    // expected
                }
            }
        }

        let canonical_base = vault
            .path
            .canonicalize()
            .context("Failed to canonicalize vault path")?;
        let full_path = vault.path.join(&file_to_sync.path);
        let canonical_full = if full_path.exists() {
            full_path.canonicalize()?
        } else {
            let parent = full_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path has no parent"))?;
            fs::create_dir_all(parent)?;
            let canonical_parent = parent.canonicalize()?;
            let filename = full_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Path has no filename"))?;
            canonical_parent.join(filename)
        };

        if !canonical_full.starts_with(&canonical_base) {
            anyhow::bail!("Path escapes vault directory: {:?}", file_to_sync.path);
        }

        network::send_file_request(&mut send, &file_to_sync.uuid).await?;
        let file_contents = network::receive_file_contents(&mut recv).await?;
        let temp_path = canonical_full.with_extension("tmp");
        fs::write(&temp_path, file_contents)?;
        fs::rename(&temp_path, &canonical_full)?;
        transfer_record.record_file_complete(RecentFile {
            uuid: file_to_sync.uuid,
            filename: file_to_sync.path.to_string_lossy().to_string(),
            timestamp: file_to_sync.modified,
        });
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
    contacts_to_send: Vec<Contact>,
    remote_endpoint_id: iroh::PublicKey,
    alpn: &[u8],
) -> Result<()> {
    let (secret_key, _) = vault.device_secret_key()?;
    if remote_endpoint_id == secret_key.public() {
        return Ok(());
    }

    let Ok(mut transfer_record) = SyncStatusRecord::start(
        vault.base_path(),
        remote_endpoint_id,
        sync_type,
        SyncDirection::Outbound,
    ) else {
        anyhow::bail!("could not create log record");
    };

    let conn = endpoint
        .connect(remote_endpoint_id, alpn)
        .await
        .context("Failed to connect to remote device")?;

    // Calling open_bi then waiting on the RecvStream without writing anything
    // to SendStream will never succeed.
    let (mut send, mut recv) = conn.open_bi().await?;

    if let Ok(Some(user_record)) = vault.user_read() {
        let user_record_bytes = serde_json::to_vec(&user_record)?;
        network::send_bytes(&mut send, &user_record_bytes).await?;
    } else {
        anyhow::bail!("cannot send files without a user record");
    }

    let contacts_bytes =
        serde_json::to_vec(&contacts_to_send).context("Failed to serialize contacts for mirror")?;
    network::send_bytes(&mut send, &contacts_bytes).await?;

    let serialised_manifest =
        serde_json::to_vec(&manifest).context("Failed to serialize manifest")?;
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

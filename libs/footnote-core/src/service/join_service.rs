use anyhow::Result;
use iroh::Endpoint;
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;

use crate::model::{
    contact::Contact,
    user::LocalUser,
    vault::{Vault, VaultState},
};

const ALPN_VAULT_JOIN: &[u8] = b"footnote/vault-join";

#[derive(Debug, Clone)]
pub enum JoinEvent {
    Listening { join_url: String },
    Success,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct UserRecordMessage {
    contact_json: String,
}

/// Join service allows a secondary device to join a vault on the primary.
pub struct JoinService;

impl JoinService {
    /// Creates an ephemeral endpoint and listens for a connection from a primary device.
    /// When connected, receives the complete user record and writes it to disk.
    pub async fn listen(vault: &Vault, cancel: CancellationToken) -> Result<Receiver<JoinEvent>> {
        if vault.state_read()? != VaultState::StandAlone {
            anyhow::bail!("A device must be in stand-alone mode to join a device group");
        }

        tracing::info!("generating iroh address for this device");
        let secret_key = iroh::SecretKey::generate(&mut rand::rng());

        tracing::info!("creating endpoint");
        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN_VAULT_JOIN.to_vec()])
            .bind()
            .await?;
        let endpoint_id = secret_key.public();
        let join_url = format!("footnote+pair://{}", endpoint_id);

        let (tx, rx) = mpsc::channel(32);
        let _ = tx.send(JoinEvent::Listening { join_url }).await;

        let vault_path = vault.path.to_path_buf();

        tokio::spawn(async move {
            tokio::select! {
                _ = cancel.cancelled() => { return; }
                maybe_incoming = endpoint.accept() => {
                    if let Some(incoming) = maybe_incoming {
                        match async {
                            tracing::info!("received incoming connection");
                            let conn = incoming.accept()?.await?;
                            tracing::info!("opened bi-directional stream");
                            let (mut send, mut recv) = conn.accept_bi().await.anyerr()?;

                            let message_bytes = recv.read_to_end(100000).await.anyerr()?;
                            tracing::info!("recv'd user record");
                            let message: UserRecordMessage = serde_json::from_slice(&message_bytes)?;
                            let contact_record = Contact::from_json(&message.contact_json)?;
                            contact_record.verify()?;
                            tracing::info!("validated user record");

                            let device_name = contact_record.devices
                                .iter()
                                .find(|d| d.iroh_endpoint_id == secret_key.public().to_string())
                                .map(|d| d.name.clone())
                                .ok_or_else(|| anyhow::anyhow!("Device not found in user record"))?;

                            let footnotes_dir = vault_path.join(".footnote");
                            let device_key_file = footnotes_dir.join("device_key");
                            let encoded_key = hex::encode(secret_key.to_bytes());
                            let device_line = format!("{} {}", encoded_key, device_name);
                            tracing::info!("writing my device info to disk");
                            fs::write(&device_key_file, device_line)?;

                            let user_file = footnotes_dir.join("user.json");
                            contact_record.to_file(user_file)?;

                            tracing::info!("sending ack");
                            let ack = b"OK";
                            send.write_all(ack).await.anyerr()?;
                            tracing::info!("closing send side");
                            send.finish().anyerr()?;
                            send.stopped().await.anyerr()?;
                            //conn.closed().await;
                            tracing::info!("complete!");
                            anyhow::Ok(())
                        }
                        .await
                        {
                            Ok(_) => {
                                let _ = tx.send(JoinEvent::Success).await;
                            }
                            Err(e) => {
                                let _ = tx.send(JoinEvent::Error(e.to_string())).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Connects to a listening device and sends the complete user record with the new device included.
    pub async fn join(vault: &Vault, connection_string: &str, device_name: &str) -> Result<()> {
        tracing::info!(
            "connecting to new device {} at {}",
            device_name,
            connection_string
        );

        let endpoint_id = parse_connection_string(connection_string)?;
        let (secret_key, _) = vault.device_secret_key()?;

        tracing::info!("build endpoint");
        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .bind()
            .await?;
        tracing::info!("connect");
        let conn = endpoint.connect(endpoint_id, ALPN_VAULT_JOIN).await?;
        tracing::info!("open bi direction stream");
        let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

        let local_user = LocalUser::new(&vault.path)?;
        let updated_contact_record =
            local_user.bless_remote_device(device_name, &endpoint_id.to_string())?;
        let message = UserRecordMessage {
            contact_json: serde_json::to_string(&updated_contact_record)?,
        };
        let message_bytes = serde_json::to_vec(&message)?;

        tracing::info!("sending user record");
        send.write_all(&message_bytes).await.anyerr()?;
        tracing::info!("closing send stream");
        send.finish().anyerr()?;

        tracing::info!("waiting for ack");
        let mut ack_buf = [0u8; 2];
        recv.read_exact(&mut ack_buf).await.anyerr()?;
        if &ack_buf != b"OK" {
            anyhow::bail!("Did not receive acknowledgment from device");
        }
        conn.closed().await;
        tracing::info!(
            "connected to new device {} at {}",
            device_name,
            connection_string
        );

        Ok(())
    }
}

fn parse_connection_string(conn_str: &str) -> Result<iroh::PublicKey> {
    let conn_str = conn_str.trim();

    if !conn_str.starts_with("footnote+pair://") {
        anyhow::bail!("Invalid connection string. Expected format: footnote+pair://endpoint-id");
    }

    let endpoint_str = conn_str.strip_prefix("footnote+pair://").unwrap();

    let endpoint_id: iroh::PublicKey = endpoint_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid endpoint ID"))?;

    Ok(endpoint_id)
}

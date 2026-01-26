use anyhow::Result;
use iroh::Endpoint;
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::model::{contact::Contact, user::LocalUser, vault::Vault};

const ALPN_VAULT_JOIN: &[u8] = b"footnote/vault-join";

#[derive(Debug, Clone)]
pub enum JoinEvent {
    Listening { join_url: String },
    Success,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceJoinRequest {
    device_name: String,
    iroh_endpoint_id: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceJoinResponse {
    contact_json: String,
}

/// Join service allows a secondary device to join a vault on the primary.
pub struct JoinService;

impl JoinService {
    /// put this device into join listen mode. an iroh url and join code will be
    /// returned that the joiner is expected to connect to and present.
    pub async fn listen(vault: &Vault, cancel: CancellationToken) -> Result<Receiver<JoinEvent>> {
        if vault.is_primary_device()? {
            anyhow::bail!(
                "This device is already in a device group. Listen for an invite from the new device."
            );
        }

        let (tx, rx) = mpsc::channel(32);
        let join_token = Uuid::new_v4().to_string();
        let (secret_key, _) = vault.device_secret_key()?;

        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN_VAULT_JOIN.to_vec()])
            .bind()
            .await?;

        let endpoint_id = secret_key.public();
        let join_url = format!("iroh://{}?token={}", endpoint_id, join_token);

        let _ = tx.send(JoinEvent::Listening { join_url }).await;

        let vault_path = vault.path.to_path_buf();

        tokio::spawn(async move {
            tokio::select! {
                _ = cancel.cancelled() => { return; }
                maybe_incoming = endpoint.accept() => {
                    if let Some(incoming) = maybe_incoming {
                        match async {
                            let conn = incoming.accept()?.await?;
                            let (mut send, mut recv) = conn.accept_bi().await.anyerr()?;
                            let request_bytes = recv.read_to_end(10000).await.anyerr()?;
                            let request: DeviceJoinRequest = serde_json::from_slice(&request_bytes)?;

                            if request.token != join_token {
                                anyhow::bail!("Invalid token. Authorization failed.");
                            }

                            let device_name = request.device_name.clone();
                            let iroh_endpoint_id = request.iroh_endpoint_id;
                            let local_user = LocalUser::new(&vault_path)?;
                            let contact_record =
                                local_user.bless_remote_device(&device_name, &iroh_endpoint_id)?;

                            let response = DeviceJoinResponse {
                                contact_json: serde_json::to_string(&contact_record)?,
                            };

                            let response_bytes = serde_json::to_vec(&response)?;
                            send.write_all(&response_bytes).await.anyerr()?;
                            send.finish().anyerr()?;

                            conn.closed().await;

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

    pub async fn join(vault: &Vault, connection_string: &str) -> Result<()> {
        let (endpoint_id, token) = parse_connection_string(connection_string)?;
        let (secret_key, device_name) = vault.device_secret_key()?;

        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .bind()
            .await?;

        let conn = endpoint.connect(endpoint_id, ALPN_VAULT_JOIN).await?;
        let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

        let request = DeviceJoinRequest {
            device_name: device_name.to_string(),
            iroh_endpoint_id: secret_key.public().to_string(),
            token,
        };

        let request_bytes = serde_json::to_vec(&request)?;
        send.write_all(&request_bytes).await.anyerr()?;
        send.finish().anyerr()?;

        let response_bytes = recv.read_to_end(100000).await.anyerr()?;
        let response: DeviceJoinResponse = serde_json::from_slice(&response_bytes)?;

        let contact_record = Contact::from_json(&response.contact_json)?;
        contact_record.verify()?;

        let footnotes_dir = vault.path.join(".footnote");
        let contact_file = footnotes_dir.join("user.json");
        contact_record.to_file(contact_file)?;

        Ok(())
    }
}

fn parse_connection_string(conn_str: &str) -> Result<(iroh::PublicKey, String)> {
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

    let query = parts[1];
    let token = query
        .split('&')
        .find(|s| s.starts_with("token="))
        .and_then(|s| s.strip_prefix("token="))
        .ok_or_else(|| anyhow::anyhow!("Token not found in connection string"))?
        .to_string();

    Ok((endpoint_id, token))
}

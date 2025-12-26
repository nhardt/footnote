use anyhow::{Context, Result};
use iroh::endpoint::Connection;
use iroh::Endpoint;
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;

use crate::model::vault::Vault;
use crate::util::manifest;
use crate::util::transfer;

pub const ALPN_REPLICA: &[u8] = b"footnote/replica";

#[derive(Debug, Clone)]
pub enum ReplicaEvent {
    Listening { endpoint_id: String },
    Received { from_device: String },
    Stopped,
    Error(String),
}

pub struct ReplicaService;

impl ReplicaService {
    pub async fn listen(vault: &Vault) -> Result<(Receiver<ReplicaEvent>, CancellationToken)> {
        let (tx, rx) = mpsc::channel(32);
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();

        let (secret_key, _) = vault.device_secret_key()?;
        let endpoint_id = secret_key.public();

        let vault_path = vault.path.to_path_buf();

        tokio::spawn(async move {
            let endpoint_result = Endpoint::builder()
                .secret_key(secret_key)
                .alpns(vec![ALPN_REPLICA.to_vec()])
                .bind()
                .await;

            let endpoint = match endpoint_result {
                Ok(ep) => ep,
                Err(e) => {
                    let _ = tx.send(ReplicaEvent::Error(e.to_string())).await;
                    return;
                }
            };

            let _ = tx
                .send(ReplicaEvent::Listening {
                    endpoint_id: endpoint_id.to_string(),
                })
                .await;

            loop {
                tokio::select! {
                    Some(incoming) = endpoint.accept() => {
                        let mut accepting = match incoming.accept() {
                            Ok(a) => a,
                            Err(e) => {
                                let _ = tx.send(ReplicaEvent::Error(format!("Accept error: {}", e))).await;
                                continue;
                            }
                        };

                        let alpn = match accepting.alpn().await {
                            Ok(a) => a,
                            Err(e) => {
                                let _ = tx.send(ReplicaEvent::Error(format!("ALPN error: {}", e))).await;
                                continue;
                            }
                        };

                        let conn = match accepting.await {
                            Ok(c) => c,
                            Err(e) => {
                                let _ = tx.send(ReplicaEvent::Error(format!("Connection error: {}", e))).await;
                                continue;
                            }
                        };

                        if alpn == ALPN_REPLICA {
                            let remote_id = conn.remote_id();
                            let vault = match Vault::new(&vault_path) {
                                Ok(v) => v,
                                Err(e) => {
                                    let _ = tx.send(ReplicaEvent::Error(format!("Vault error: {}", e))).await;
                                    continue;
                                }
                            };

                            let device_name = match vault.owned_device_endpoint_to_name(&remote_id) {
                                Ok(name) => {
                                    let _ = tx.send(ReplicaEvent::Received { from_device: name.clone() }).await;
                                    name
                                }
                                Err(e) => {
                                    let _ = tx.send(ReplicaEvent::Error(format!("Not my device: {}", e))).await;
                                    continue;
                                }
                            };

                            let conn_clone = conn.clone();
                            tokio::spawn(async move {
                                if let Err(e) = transfer::receive_replication(&vault, conn_clone).await {
                                    eprintln!("Error handling replica from {}: {:?}", device_name, e);
                                }
                            });
                        }
                    }
                    _ = cancel_clone.cancelled() => {
                        let _ = tx.send(ReplicaEvent::Stopped).await;
                        break;
                    }
                }
            }
        });

        Ok((rx, cancel_token))
    }

    pub async fn push(vault: &Vault, device_name: &str) -> Result<()> {
        let endpoint_str = vault.owned_device_name_to_endpoint(device_name)?;
        let endpoint_id = endpoint_str.parse::<iroh::PublicKey>()?;

        let manifest =
            manifest::create_manifest_full(&vault.path).context("Failed to create manifest")?;

        transfer::replicate_to_target(vault, manifest, endpoint_id, ALPN_REPLICA).await?;

        Ok(())
    }
}

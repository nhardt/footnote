use anyhow::Result;
use iroh::{endpoint::Connection, Endpoint};
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{model::vault::Vault, util::transfer};

/// iroh listener for mirror and share protocols
pub struct SyncService;

const ALPN_SHARE: &[u8] = b"footnote/share";
const ALPN_REPLICA: &[u8] = b"footnote/mirror";

impl SyncService {
    pub async fn listen(vault_path: &Path, cancel: CancellationToken) -> Result<()> {
        let vault = Vault::new(vault_path)?;
        let vault_path = vault_path.to_path_buf();
        let (secret_key, _) = vault.device_secret_key()?;

        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN_REPLICA.to_vec(), ALPN_SHARE.to_vec()])
            .bind()
            .await?;

        println!("Listening on endpoint: {}", secret_key.public());

        tokio::spawn(async move {
            tokio::select! {
                // tokio::select implicitely awaits each of these
                _ = cancel.cancelled() => { return; }

                maybe_incoming = endpoint.accept() => {
                    if let Some(incoming) = maybe_incoming {
                        match async {
                            let connection = incoming.await?;
                            let alpn = connection.alpn();
                            match alpn {
                                ALPN_REPLICA => {
                                    Self::handle_replica(&vault_path, connection).await?;
                                }
                                ALPN_SHARE => {
                                    Self::handle_share(&vault_path, connection).await?;
                                }
                                _ => {
                                    anyhow::bail!(format!("Protocol disabled or unknown: {:?}", alpn));
                                }
                            }
                            anyhow::Ok(())
                        }.await {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("failed to handle request {}", e.to_string());
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }

    async fn handle_replica(vault_path: &Path, connection: Connection) -> Result<()> {
        let vault = Vault::new(vault_path)?;
        let remote_id = connection.remote_id();
        let device_name = vault.owned_device_endpoint_to_name(&remote_id)?;
        transfer::receive_replication(&vault, connection).await?;
        eprintln!("succesfully handled replicate request from {}", device_name);
        Ok(())
    }

    async fn handle_share(vault_path: &Path, connection: Connection) -> Result<()> {
        let vault = Vault::new(vault_path)?;
        let remote_id = connection.remote_id();
        let contact = vault.find_contact_by_endpoint(&remote_id)?;
        transfer::receive_share(&vault, &contact.nickname, connection).await?;
        eprintln!(
            "succesfully recieved shared files from {}",
            contact.nickname
        );
        Ok(())
    }
}

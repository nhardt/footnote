use anyhow::{Context, Result};
use iroh::endpoint::Connection;
use iroh::Endpoint;
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;

use crate::model::vault::Vault;
use crate::util::manifest;
use crate::util::transfer;

pub const ALPN_SHARE: &[u8] = b"footnote/sync";

#[derive(Debug, Clone)]
pub enum ShareEvent {
    Listening { endpoint_id: String },
    Received { from_device: String },
    Stopped,
    Error(String),
}

pub struct ShareService;

impl ShareService {
    pub async fn listen(vault: &Vault) -> Result<()> {
        let (secret_key, _) = vault.device_secret_key()?;
        let vault_path = vault.path.to_path_buf();

        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN_SHARE.to_vec()])
            .bind()
            .await?;

        println!("Listening on endpoint: {}", secret_key.public());

        while let Some(incoming) = endpoint.accept().await {
            let vault_path = vault_path.clone();
            tokio::spawn(async move {
                let connection = match incoming.await {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Connection error: {}", e);
                        return;
                    }
                };

                let vault = match Vault::new(&vault_path) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Vault error: {}", e);
                        return;
                    }
                };

                let remote_id = connection.remote_id();
                let nickname = match vault.find_contact_by_endpoint(&remote_id) {
                    Ok(contact) => {
                        println!("Receiving from device: {}", contact.nickname);
                        contact.nickname
                    }
                    Err(e) => {
                        eprintln!("unknown device: {}", e);
                        return;
                    }
                };

                if let Err(e) = transfer::receive_share(&vault, &nickname, connection).await {
                    eprintln!("Error handling share from {}: {:?}", nickname, e);
                }
            });
        }

        Ok(())
    }

    pub async fn share_with(vault: &Vault, endpoint: Endpoint, nickname: &str) -> Result<()> {
        let endpoint_id = match vault.find_primary_device_by_nickname(nickname) {
            Ok(eid) => {
                println!("will share with {} via {}", nickname, eid.to_string());
                eid
            }
            Err(e) => {
                eprintln!("error getting primary device: {}", e);
                anyhow::bail!("no primary device for nickname")
            }
        };
        let manifest = manifest::create_manifest_for_share(&vault.path, nickname)
            .context("Failed to create manifest for sharing")?;

        transfer::replicate_to_target(vault, endpoint, manifest, endpoint_id, ALPN_SHARE).await?;

        Ok(())
    }
}

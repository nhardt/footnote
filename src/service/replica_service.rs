use anyhow::{Context, Result};
use iroh::endpoint::Connection;
use iroh::Endpoint;
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;

use crate::model::vault::Vault;
use crate::util::manifest;
use crate::util::transfer;

pub const ALPN_REPLICA: &[u8] = b"footnote/sync";

#[derive(Debug, Clone)]
pub enum ReplicaEvent {
    Listening { endpoint_id: String },
    Received { from_device: String },
    Stopped,
    Error(String),
}

pub struct ReplicaService;

impl ReplicaService {
    pub async fn listen(vault: &Vault) -> Result<()> {
        let (secret_key, _) = vault.device_secret_key()?;
        let vault_path = vault.path.to_path_buf();

        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN_REPLICA.to_vec()])
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
                let device_name = match vault.owned_device_endpoint_to_name(&remote_id) {
                    Ok(name) => {
                        println!("Receiving from device: {}", name);
                        name
                    }
                    Err(e) => {
                        eprintln!("Not my device: {}", e);
                        return;
                    }
                };

                if let Err(e) = transfer::receive_replication(&vault, connection).await {
                    eprintln!("Error handling replica from {}: {:?}", device_name, e);
                }
            });
        }

        Ok(())
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

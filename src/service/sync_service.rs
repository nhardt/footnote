use crate::{
    model::vault::Vault,
    util::{manifest, transfer},
};
use anyhow::{Context, Result};
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

pub struct SyncService;

const ALPN_SYNC: &[u8] = b"footnote/sync";

impl SyncService {
    pub async fn listen(vault: Vault, endpoint: Endpoint, cancel: CancellationToken) -> Result<()> {
        let (secret_key, _) = vault.device_secret_key()?;

        tracing::info!("Listening on endpoint: {}", secret_key.public());

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    tracing::info!("Listening cancelled on endpoint: {}", secret_key.public());
                    break;
                }

                maybe_incoming = endpoint.accept() => {
                    if let Some(incoming) = maybe_incoming {
                        tracing::info!("Woke up endpoint: {}", secret_key.public());
                        match async {
                            let connection = incoming.await?;
                            let remote_id = connection.remote_id();
                            tracing::info!( "succesfully connection from {}", remote_id);

                            if let Ok(contact) = vault.find_contact_by_endpoint(&remote_id) {
                                tracing::info!( "found contact {} from endpoint {}", contact.nickname, remote_id);
                                transfer::receive_share(&vault, &contact.nickname, connection.clone()).await?;
                                tracing::info!( "succesfully recieved shared files from {}", contact.nickname);
                                return Ok(());
                            }

                            if let Ok(device_name) = vault.owned_device_endpoint_to_name(&remote_id) {
                                tracing::info!("found our own device {} from endpoint {}", device_name, remote_id);
                                transfer::receive_replication(&vault, connection).await?;
                                tracing::info!("succesfully handled replicate request from {} on {}", device_name, remote_id);
                                return Ok(());
                            }
                            anyhow::bail!("failed to handle incoming connection")
                        }.await {
                            Ok(_) => {
                                tracing::info!("Handling incoming connection");
                            }
                            Err(e) => {
                                tracing::info!("failed to handle request {}", e.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn share_to_device(vault: &Vault, endpoint: Endpoint, nickname: &str) -> Result<()> {
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

        transfer::replicate_to_target(vault, endpoint, manifest, endpoint_id, ALPN_SYNC).await?;

        Ok(())
    }

    pub async fn mirror_to_device(
        vault: &Vault,
        endpoint: Endpoint,
        device_name: &str,
    ) -> Result<()> {
        let endpoint_str = vault.owned_device_name_to_endpoint(device_name)?;
        let endpoint_id = endpoint_str.parse::<iroh::PublicKey>()?;

        let manifest =
            manifest::create_manifest_full(&vault.path).context("Failed to create manifest")?;

        transfer::replicate_to_target(vault, endpoint, manifest, endpoint_id, ALPN_SYNC).await?;

        Ok(())
    }
}

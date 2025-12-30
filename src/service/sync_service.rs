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

pub struct SyncService;

const ALPN_SYNC: &[u8] = b"footnote/sync";

impl SyncService {
    pub async fn listen(vault_path: &Path, cancel: CancellationToken) -> Result<()> {
        let vault = Vault::new(vault_path)?;
        let vault_path = vault_path.to_path_buf();
        let (secret_key, _) = vault.device_secret_key()?;

        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN_SYNC.to_vec()])
            .bind()
            .await?;

        println!("Listening on endpoint: {}", secret_key.public());

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => { return; }

                    maybe_incoming = endpoint.accept() => {
                        if let Some(incoming) = maybe_incoming {
                            match async {
                                let connection = incoming.await?;

                                let vault = Vault::new(&vault_path)?;
                                let remote_id = connection.remote_id();

                                if let Ok(contact) = vault.find_contact_by_endpoint(&remote_id) {
                                    transfer::receive_share(&vault, &contact.nickname, connection.clone()).await?;
                                    eprintln!( "succesfully recieved shared files from {}", contact.nickname);
                                    return Ok(());
                                }
                                if let Ok(device_name) = vault.owned_device_endpoint_to_name(&remote_id) {
                                    transfer::receive_replication(&vault, connection).await?;
                                    eprintln!("succesfully handled replicate request from {}", device_name);
                                    return Ok(());
                                }
                                anyhow::bail!("failed to handle incoming connection")
                            }.await {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("failed to handle request {}", e.to_string());
                                }
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use ed25519_dalek::SigningKey;
use iroh::{Endpoint, SecretKey};
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc::{self, Receiver};
use uuid::Uuid;

use crate::model::user::LocalUser;

//// Device Auth Protocol
const ALPN_DEVICE_AUTH: &[u8] = b"footnote/device-auth";
#[derive(Debug, Clone)]
pub enum DeviceAuthEvent {
    Listening { join_url: String },
    Connecting,
    ReceivedRequest { device_name: String },
    Verifying,
    Success { device_name: String },
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
//// End Device Auth Protocol

pub struct Vault {
    path: PathBuf,
}

/// inside a footnote vault:
///
/// .footnote/
///    id_key               : private key that signs device record, primary only
///    device_key           : private key specific to this device
impl Vault {
    /// called on the first device when creating a new vault
    pub fn create_primary(path: PathBuf, username: &str, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        LocalUser::create_local_user_record(&v.path, username, device_name)?;
        Ok(v)
    }

    /// called on non-primary device to put vault into state where it's ready to
    /// join
    pub fn create_secondary(path: PathBuf, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        v.create_device_key(device_name)?;
        Ok(v)
    }

    /// Call on an existing vault to use vault API
    pub fn new(path: PathBuf) -> Result<Self> {
        let v = Self { path };
        Ok(v)
    }

    fn create_directory_structure(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        fs::create_dir_all(&footnotes_dir)?;
        Ok(())
    }

    fn device_secret_key(&self) -> Result<iroh::SecretKey> {
        let footnotes_dir = self.path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let content = fs::read_to_string(device_key_file)?;
        let (encoded_key, name) = content.split_once(' ')?;
        let key_vec: Vec<u8> = general_purpose::STANDARD.decode(encoded_key)?;
        let key_array: [u8; 32] = key_vec
            .try_into()
            .map_err(|_| anyhow::anyhow!("Device key must be exactly 32 bytes"))?;
        let secret_key = iroh::SecretKey::from_bytes(&key_array);
        Ok(secret_key)
    }

    /// put this primary device into join listen mode. an iroh url and join code will be
    /// returned that the joiner is expected to connect to and present.
    pub async fn join_listen(
        &self,
        device_name: &str,
    ) -> anyhow::Result<Receiver<DeviceAuthEvent>> {
        if !self.is_primary_device()? {
            anyhow::bail!(
            "This device is not marked as primary. Only the primary device can create join URLs.\n\
            Run this command on your primary device."
        );
        }
        let (tx, rx) = mpsc::channel(32);
        let token = Uuid::new_v4().to_string();
        let secret_key = self.device_secret_key()?;

        // Create Iroh endpoint
        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN_DEVICE_AUTH.to_vec()])
            .bind()
            .await?;

        let endpoint_id = secret_key.public();
        let join_url = format!("iroh://{}?token={}", endpoint_id, token);

        // Send initial listening event with URL
        let _ = tx
            .send(DeviceAuthEvent::Listening {
                join_url: join_url.clone(),
            })
            .await;

        tokio::spawn(async move {
            if let Some(incoming) = endpoint.accept().await {
                let _ = tx.send(DeviceAuthEvent::Connecting).await;

                match async {
                    let conn = incoming.accept()?.await?;
                    let (mut send, mut recv) = conn.accept_bi().await.anyerr()?;
                    let request_bytes = recv.read_to_end(10000).await.anyerr()?;
                    let request: DeviceJoinRequest = serde_json::from_slice(&request_bytes)?;

                    let _ = tx
                        .send(DeviceAuthEvent::ReceivedRequest {
                            device_name: request.device_name.clone(),
                        })
                        .await;

                    if request.token != token {
                        anyhow::bail!("Invalid token. Authorization failed.");
                    }

                    let _ = tx.send(DeviceAuthEvent::Verifying).await;

                    // we were waiting for a connection from a device with our
                    // auth code. we recieved it. we could ask the user to type
                    // the device on both sides as well but the code+url is sufficienct
                    //
                    // device_name: request.device_name.clone(),
                    // iroh_endpoint_id: request.iroh_endpoint_id,
                    //
                    // let contact = Contact::from_disk(self.vault_path)?;
                    // contact.create_device(request.device_name, iroh_endpoint_id)
                    // contact.save()
                    // contact.to_json()

                    let response = DeviceJoinResponse {
                        contact_json: serde_json::to_string(&contact_record)?,
                    };

                    let response_bytes = serde_json::to_vec(&response)?;
                    send.write_all(&response_bytes).await.anyerr()?;
                    send.finish().anyerr()?;

                    conn.closed().await;

                    Ok::<_, anyhow::Error>(request.device_name.clone())
                }
                .await
                {
                    Ok(device_name) => {
                        let _ = tx.send(DeviceAuthEvent::Success { device_name }).await;
                    }
                    Err(e) => {
                        let _ = tx.send(DeviceAuthEvent::Error(e.to_string())).await;
                    }
                }
            }
        });

        Ok(rx)
    }

    pub fn is_primary_device(&self) -> anyhow::Result<bool> {
        Ok(self.path.join(".footnote").join("id_key").exists())
    }
}

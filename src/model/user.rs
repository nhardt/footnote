use crate::model::contact::Contact;
use crate::model::device::Device;
use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use ed25519_dalek::SigningKey;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LocalUser {
    pub vault_path: PathBuf,
    pub devices: Vec<Device>,
}

impl LocalUser {
    pub fn new(path: PathBuf) -> Result<Self> {
        let v = Self {
            vault_path: path,
            devices: [].to_vec(),
        };
        Ok(v)
    }

    pub fn id_key(&self) -> Result<ed25519_dalek::SigningKey> {
        let footnotes_dir = self.vault_path.join(".footnote");
        let id_key_file = footnotes_dir.join("id_key");
        let content = fs::read_to_string(id_key_file)?;
        let (encoded_key, name) = content.split_once(' ')?;
        let key_vec: Vec<u8> = general_purpose::STANDARD.decode(encoded_key)?;
        let key_array: [u8; 32] = key_vec
            .try_into()
            .map_err(|_| anyhow::anyhow!("Device key must be exactly 32 bytes"))?;
        let secret_key = SigningKey::from_bytes(&key_array);
        Ok(secret_key)
    }

    pub fn id_key_pub(&self) -> Result<ed25519_dalek::VerifyingKey> {
        Ok(self.id_key()?.verifying_key())
    }

    pub fn device_key(&self) -> Result<(iroh::SecretKey, String)> {
        let footnotes_dir = self.vault_path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let content = fs::read_to_string(device_key_file)?;
        let (encoded_key, device_name) = content.split_once(' ')?;
        let key_vec: Vec<u8> = general_purpose::STANDARD.decode(encoded_key)?;
        let key_array: [u8; 32] = key_vec
            .try_into()
            .map_err(|_| anyhow::anyhow!("Device key must be exactly 32 bytes"))?;
        let secret_key = iroh::SecretKey::from_bytes(&key_array);
        Ok((secret_key, device_name.to_string()))
    }
    pub fn device_key_pub(&self) -> Result<(iroh::PublicKey, String)> {
        let (device_key, device_name) = self.device_key()?;
        Ok((device_key.public(), device_name))
    }

    pub fn bless_remote_device(device_name: String, iroh_endpoint: String) {}

    pub fn to_contact(&self) -> Result<Contact> {
        let signable = SignableContact {
            username: &self.username,
            identity_verifying_key: &self.id_key_pub(),
            devices: &self.devices,
            updated_at: self.updated_at,
        };
        let message = serde_json::to_string(&signable)?;
        let signature = signing_key.sign(message.as_bytes());
        self.signature = hex::encode(signature.to_bytes());
        Ok(())
    }

    pub fn from_disk(vault_path: impl AsRef<Path>) -> Result<Self> {
        let local_user_file = vault_path.join(".footnote")?.join("user.json")?;
        let file_contents = fs::read_to_string(local_user_file);
        match (file_contents) {
            Ok(contents) => from_json(contents),
            Err(error) => {}
        }
    }
}

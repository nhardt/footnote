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

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    fn create_test_signing_key() -> SigningKey {
        let mut csprng = OsRng;
        SigningKey::generate(&mut csprng)
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();
        assert!(contact.verify().unwrap());
    }

    #[test]
    fn test_nickname_not_verified() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "Alice W.".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();

        // Changing nickname should not break verification
        contact.nickname = "Different Name".to_string();
        assert!(contact.verify().unwrap());
    }

    #[test]
    fn test_verify_fails_with_wrong_signature() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            devices: vec![],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();

        contact.signature = "0000".to_string();
        assert!(!contact.verify().unwrap());
    }

    #[test]
    fn test_is_valid_successor() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();
        let master_key = hex::encode(verifying_key.to_bytes());

        let mut contact_v1 = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };
        contact_v1.sign(&signing_key).unwrap();

        let mut contact_v2 = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            devices: vec![
                Device::new("laptop".to_string(), "abc123".to_string()),
                Device::new("phone".to_string(), "def456".to_string()),
            ],
            updated_at: LamportTimestamp(2000),
            signature: String::new(),
        };
        contact_v2.sign(&signing_key).unwrap();

        assert!(contact_v2.is_valid_successor(&contact_v1).unwrap());
        assert!(!contact_v1.is_valid_successor(&contact_v2).unwrap());
    }

    #[test]
    fn test_json_round_trip() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "Alice W.".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };
        contact.sign(&signing_key).unwrap();

        let json = contact.to_json().unwrap();
        let loaded = Contact::from_json(&json).unwrap();

        assert_eq!(loaded.username, contact.username);
        assert_eq!(loaded.devices.len(), contact.devices.len());
        assert!(loaded.verify().unwrap());
    }
}

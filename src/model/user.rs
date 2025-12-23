use crate::model::contact::{Contact, SignableContact};
use crate::model::device::Device;
use crate::model::lamport_timestamp::LamportTimestamp;
use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use ed25519_dalek::SigningKey;
use std::fs;
use std::path::{Path, PathBuf};

use rand_core::OsRng;
pub struct LocalUser {
    pub vault_path: PathBuf,
    pub devices: Vec<Device>,
}

impl LocalUser {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let v = Self {
            vault_path: path.to_path_buf(),
            devices: [].to_vec(),
        };
        Ok(v)
    }

    pub fn create_local_user_record(
        vault_path: &Path,
        username: &str,
        device_name: &str,
    ) -> Result<()> {
        let local_user = LocalUser::new(vault_path)?;
        local_user.create_id_key(username)?;
        local_user.create_device_key(device_name)?;

        let (id_signing_key, _) = local_user.id_key()?;
        let (device_signing_key, device_name) = local_user.device_key()?;

        let local_device = Device {
            iroh_endpoint_id: device_signing_key.public().to_string(),
            name: device_name,
        };
        let local_user_contact_record = Contact::new_local_user_record(
            username,
            id_signing_key.verifying_key().to_bytes(),
            device_name,
            id_signing_key,
        );

        Ok(())
    }

    /// the id key is the master public key for the vault. it's generated and
    /// stored on the primary device. this key is used to sign the contact
    /// record, and represents a user's stable identity.
    fn create_id_key(&self, username: &str) -> anyhow::Result<()> {
        let footnotes_dir = self.vault_path.join(".footnote");
        let id_key_file = footnotes_dir.join("id_key");
        let mut csprng = OsRng;
        let id_key = SigningKey::generate(&mut csprng);
        let encoded_key = hex::encode(id_key.to_bytes());
        let id_line = format!("{} {}", encoded_key, username);
        fs::write(&id_key_file, id_line)?;
        Ok(())
    }

    /// the device signing key is generated and stored local to each device. it
    /// is used in establishing verified connections between devices via iroh.
    fn create_device_key(&self, device_name: &str) -> anyhow::Result<()> {
        let footnotes_dir = self.vault_path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let device_key = iroh::SecretKey::generate(&mut rand::rng());
        let encoded_key = hex::encode(device_key.to_bytes());
        let device_line = format!("{} {}", encoded_key, device_name);
        fs::write(&device_key_file, device_line)?;
        Ok(())
    }

    pub fn id_key(&self) -> Result<(ed25519_dalek::SigningKey, String)> {
        let footnotes_dir = self.vault_path.join(".footnote");
        let id_key_file = footnotes_dir.join("id_key");
        let content = fs::read_to_string(id_key_file)?;
        let (encoded_key, username) = match content.split_once(' ') {
            Some((a, b)) => (a, b),
            None => anyhow::bail!("username not found in key"),
        };
        let key_vec: Vec<u8> = hex::decode(encoded_key)?;
        let key_array: [u8; 32] = key_vec
            .try_into()
            .map_err(|_| anyhow::anyhow!("Device key must be exactly 32 bytes"))?;
        let secret_key = SigningKey::from_bytes(&key_array);
        Ok((secret_key, username.to_string()))
    }

    pub fn id_key_pub(&self) -> Result<(ed25519_dalek::VerifyingKey, String)> {
        let (private_key, username) = self.id_key()?;
        Ok((private_key.verifying_key(), username))
    }

    pub fn device_key(&self) -> Result<(iroh::SecretKey, String)> {
        let footnotes_dir = self.vault_path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let content = fs::read_to_string(device_key_file)?;
        let (encoded_key, device_name) = match content.split_once(' ') {
            Some((a, b)) => (a, b),
            None => anyhow::bail!("device record has no name"),
        };
        let key_vec: Vec<u8> = hex::decode(encoded_key)?;
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

    pub fn bless_remote_device(&self, device_name: &str, iroh_endpoint: &str) -> Result<()> {
        let local_user_file = self.vault_path.join(".footnote").join("user.json");
        let current_user_record = match Contact::from_file(&local_user_file) {
            Ok(contact) => contact,
            Err(_) => anyhow::bail!("no user file exists to bless device into"),
        };
        current_user_record.verify()?;

        let mut user_record = current_user_record.clone();
        user_record.devices.push(Device {
            name: device_name,
            iroh_endpoint_id: iroh_endpoint,
        });
        user_record.updated_at = LamportTimestamp(user_record.updated_at.as_i64());
        user_record.sign(&self.id_key()?.0);
        current_user_record.validate_successor(&user_record)?;
        user_record.to_file(&local_user_file)?;
        Ok(())
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

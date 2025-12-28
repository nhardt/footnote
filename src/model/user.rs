use crate::model::contact::Contact;
use crate::model::device::Device;
use crate::model::lamport_timestamp::LamportTimestamp;
use anyhow::Result;
use ed25519_dalek::SigningKey;
use std::fs;
use std::path::{Path, PathBuf};

use rand_core::OsRng;
pub struct LocalUser {
    pub vault_path: PathBuf,
    pub devices: Vec<Device>,
}

impl LocalUser {
    pub fn new(path: &Path) -> Result<Self> {
        let v = Self {
            vault_path: path.to_path_buf(),
            devices: Vec::new(),
        };
        Ok(v)
    }

    /// the local user record is intended to exist on the primary. the primary
    /// has a public key and username used to sign the collected device records.
    pub fn create_local_user_record(vault_path: &Path, username: &str) -> Result<()> {
        let local_user = LocalUser::new(vault_path)?;
        local_user.create_id_key(username)?;

        let (id_signing_key, _) = local_user.id_key()?;
        let (device_signing_key, device_name) = local_user.device_key()?;

        let local_device = Device {
            iroh_endpoint_id: device_signing_key.public().to_string(),
            name: device_name,
        };

        let id_public_key_str = hex::encode(id_signing_key.verifying_key().to_bytes());

        let mut local_user_contact_record = Contact::new_local_user_record(
            username,
            &id_public_key_str,
            local_device,
            &id_signing_key,
        )?;

        local_user_contact_record.sign(&id_signing_key)?;
        let local_user_file = vault_path.join(".footnote").join("user.json");
        local_user_contact_record.to_file(local_user_file)?;

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

    pub fn bless_remote_device(&self, device_name: &str, iroh_endpoint: &str) -> Result<Contact> {
        let local_user_file = self.vault_path.join(".footnote").join("user.json");
        let current_user_record = match Contact::from_file(&local_user_file) {
            Ok(contact) => contact,
            Err(_) => anyhow::bail!("no user file exists to bless device into"),
        };
        current_user_record.verify()?;

        let mut user_record = current_user_record.clone();
        user_record.devices.push(Device {
            name: device_name.to_string(),
            iroh_endpoint_id: iroh_endpoint.to_string(),
        });
        user_record.updated_at = LamportTimestamp(user_record.updated_at.as_i64());
        let (signing_key, _) = self.id_key()?;
        user_record.sign(&signing_key)?;
        user_record.is_valid_successor_of(&current_user_record)?;
        user_record.to_file(&local_user_file)?;
        Ok(user_record)
    }

    // it's potentially better to update the id record, then rebuild the record
    // from that to maintain a single source of truth. or, should update the username
    // in both places.
    pub fn update_username(&self, username: &str) -> Result<Contact> {
        let local_user_file = self.vault_path.join(".footnote").join("user.json");
        let current_user_record = match Contact::from_file(&local_user_file) {
            Ok(contact) => contact,
            Err(_) => anyhow::bail!("no user file exists to bless device into"),
        };
        current_user_record.verify()?;

        let mut user_record = current_user_record.clone();
        user_record.username = username.to_string();
        user_record.updated_at = LamportTimestamp(user_record.updated_at.as_i64());
        let (signing_key, _) = self.id_key()?;
        user_record.sign(&signing_key)?;
        user_record.is_valid_successor_of(&current_user_record)?;
        user_record.to_file(&local_user_file)?;
        Ok(user_record)
    }
}

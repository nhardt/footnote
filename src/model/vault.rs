use anyhow::Result;
use iroh::Endpoint;
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::{self, Receiver};
use uuid::Uuid;

use crate::model::{contact::Contact, note::Note, user::LocalUser};

pub struct Vault {
    pub path: PathBuf,
}

/// inside a footnote vault:
///
/// .footnote/
///    id_key               : private key that signs device record, primary only
///    device_key           : private key specific to this device
///    user.json            : signed record of the local user's devices
impl Vault {
    /// called on the first device when creating a new vault
    pub fn create_primary(path: PathBuf, username: &str, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        v.create_device_key(device_name)?;
        LocalUser::create_local_user_record(&v.path, username)?;
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
    pub fn new(path: &Path) -> Result<Self> {
        let v = Self {
            path: path.to_path_buf(),
        };
        Ok(v)
    }

    fn create_directory_structure(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        fs::create_dir_all(&footnotes_dir)?;
        Ok(())
    }

    /// the device signing key is generated and stored local to each device. it
    /// is used in establishing verified connections between devices via iroh.
    fn create_device_key(&self, device_name: &str) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let device_key = iroh::SecretKey::generate(&mut rand::rng());
        let encoded_key = hex::encode(device_key.to_bytes());
        let device_line = format!("{} {}", encoded_key, device_name);
        fs::write(&device_key_file, device_line)?;
        Ok(())
    }

    pub fn device_secret_key(&self) -> Result<(iroh::SecretKey, String)> {
        let footnotes_dir = self.path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let content = fs::read_to_string(device_key_file)?;
        let (encoded_key, device_name) = match content.split_once(' ') {
            Some((a, b)) => (a, b),
            None => anyhow::bail!("username not found in key"),
        };
        let key_vec: Vec<u8> = hex::decode(encoded_key)?;
        let key_array: [u8; 32] = key_vec
            .try_into()
            .map_err(|_| anyhow::anyhow!("Device key must be exactly 32 bytes"))?;
        let secret_key = iroh::SecretKey::from_bytes(&key_array);
        Ok((secret_key, device_name.to_string()))
    }

    pub fn is_primary_device(&self) -> anyhow::Result<bool> {
        Ok(self.path.join(".footnote").join("id_key").exists())
    }

    pub fn can_device_read_note(
        &self,
        device_endpoint: &iroh::PublicKey,
        note_path: &Path,
    ) -> Result<bool> {
        if self.owned_device_to_name(device_endpoint).is_ok() {
            return Ok(true);
        }

        let contact = match self.find_contact_by_endpoint(device_endpoint) {
            Ok(c) => c,
            Err(_) => {
                return Ok(false);
            }
        };

        let note = Note::from_path(note_path)?;

        if note.frontmatter.share_with.contains(&contact.nickname) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn owned_device_to_name(&self, endpoint_id: &iroh::PublicKey) -> anyhow::Result<String> {
        let owned_devices_record =
            Contact::from_file(self.path.join(".footnote").join("user.json"))?;

        for device in owned_devices_record.devices {
            if let Ok(device_endpoint_id) = device.iroh_endpoint_id.parse::<iroh::PublicKey>() {
                if &device_endpoint_id == endpoint_id {
                    return Ok(device.name.clone());
                }
            }
        }

        anyhow::bail!("Device is unknown")
    }

    pub fn owned_device_name_to_endpoint(&self, device_name: &str) -> anyhow::Result<String> {
        let owned_devices_record =
            Contact::from_file(self.path.join(".footnote").join("user.json"))?;

        for device in owned_devices_record.devices {
            if device.name == device_name {
                return Ok(device.iroh_endpoint_id);
            }
        }

        anyhow::bail!("Device is unknown")
    }

    pub fn find_contact_by_endpoint(&self, endpoint: &iroh::PublicKey) -> Result<Contact> {
        let contacts_dir = self.path.join(".footnote").join("contacts");

        for entry in fs::read_dir(contacts_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let contact = Contact::from_file(entry.path())?;

                for device in &contact.devices {
                    if let Ok(device_endpoint) = device.iroh_endpoint_id.parse::<iroh::PublicKey>()
                    {
                        if &device_endpoint == endpoint {
                            // note: storing the user's share name by file name
                            // would ensure locally unique names
                            return Ok(contact);
                        }
                    }
                }
            }
        }

        anyhow::bail!("No contact found with endpoint {}", endpoint)
    }
}

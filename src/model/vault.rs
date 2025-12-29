use anyhow::Result;
use iroh::Endpoint;
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::{self, Receiver};
use uuid::Uuid;

use crate::model::contact;
use crate::model::device::Device;
use crate::model::{contact::Contact, note::Note, user::LocalUser};

pub struct Vault {
    pub path: PathBuf,
}

pub enum VaultState {
    Primary,
    SecondaryJoined,
    SecondaryUnjoined,
    StandAlone,
    Uninitialized,
}

/// inside a footnote vault:
///
/// .footnote/
///    id_key               : private key that signs device record, primary only
///    device_key           : private key specific to this device
///    user.json            : signed record of the local user's devices
impl Vault {
    /// Create a vault handle
    pub fn new(path: &Path) -> Result<Self> {
        let v = Self {
            path: path.to_path_buf(),
        };
        Ok(v)
    }

    // Vault
    //
    // - by default, vault is StandAlone
    //   - Join -> Secondary: Gains user.json from primary, no id_key
    //   - ToPrimary -> Primary: Gains user.json from primary, id_key, id_key matches signing key
    // - Secondary: Terminal, can edit by hand if you know what you're doing
    // - Primary: Terminal, can edit by hand if yo know what you're doing
    // - device name update:
    //   - user.json does not exist: can edit local device name
    //   - user.json exists, we are primary: can edit device names
    //   - user.json exists, we are not primary: our local device name should be
    //     pulled from user.json, but from a "source of truth", the device can set
    //     its own name. can it be re-added to the primary.
    // - device read:
    //   - user.json exists: return all. our local device should be there
    //   - no user.json: just return our local device. it's name is editable
    //   - on vault, since it applies whether or not we are primary
    //
    // vault states:
    // - StandAlone: init this way. can join or become primary
    //
    // Merging repos with two sets of files.
    // - different filename, NBD
    // - same filename, the reqesting side will write incoming to
    //   filename-{device_name}.md (or {uuid[8..]}
    pub fn state_read(&self) -> Result<VaultState> {
        if self.path.join(".footnote").join("id_key").exists() {
            return Ok(VaultState::Primary);
        }

        if self.path.join(".footnote").join("user.json").exists() {
            return Ok(VaultState::SecondaryJoined);
        }

        if self.path.join(".footnote").join("device_key").exists() {
            return Ok(VaultState::SecondaryUnjoined);
        }

        if self.path.join(".footnote").exists() {
            return Ok(VaultState::StandAlone);
        }

        Ok(VaultState::Uninitialized)
    }

    /// called on the first device when creating a new vault
    pub fn create_primary(path: &Path, username: &str, device_name: &str) -> Result<Self> {
        let v = Self {
            path: path.to_path_buf(),
        };
        v.create_directory_structure()?;
        v.create_device_key(device_name)?;
        LocalUser::create_local_user_record(&v.path, username)?;
        Ok(v)
    }

    pub fn transition_to_primary(&self, username: &str, device_name: &str) -> Result<()> {
        match self.state_read()? {
            VaultState::Uninitialized => {
                self.create_directory_structure()?;
                self.create_device_key(device_name)?;
                LocalUser::create_local_user_record(&self.path, username)?;
            }
            VaultState::StandAlone => {
                self.create_directory_structure()?;
                self.create_device_key(device_name)?;
                LocalUser::create_local_user_record(&self.path, username)?;
            }
            VaultState::SecondaryUnjoined => {
                anyhow::bail!("Unjoined to Primary currently unsupported");
            }
            VaultState::SecondaryJoined => {
                anyhow::bail!("Unjoined to Primary currently unsupported");
            }
            VaultState::Primary => {}
        }

        Ok(())
    }

    /// called on non-primary device to put vault into state where it's ready to
    /// join
    pub fn create_secondary(path: &Path, device_name: &str) -> Result<Self> {
        let v = Self {
            path: path.to_path_buf(),
        };
        v.create_directory_structure()?;
        v.create_device_key(device_name)?;
        Ok(v)
    }

    pub fn transition_to_secondary(&self, device_name: &str) -> Result<()> {
        match self.state_read()? {
            VaultState::Uninitialized => {
                self.create_directory_structure()?;
                self.create_device_key(device_name)?;
            }
            VaultState::StandAlone => {
                self.create_directory_structure()?;
                self.create_device_key(device_name)?;
            }
            VaultState::SecondaryUnjoined => {}
            VaultState::SecondaryJoined => {}
            VaultState::Primary => {
                anyhow::bail!("Unjoined to Primary currently unsupported");
            }
        }

        Ok(())
    }

    /// called on non-primary device to put vault into state where it's ready to
    /// join
    pub fn create_standalone(path: &Path) -> Result<Self> {
        let v = Self {
            path: path.to_path_buf(),
        };
        v.create_directory_structure()?;
        Ok(v)
    }

    fn create_directory_structure(&self) -> anyhow::Result<()> {
        let footnote_dir = self.path.join(".footnote");
        fs::create_dir_all(&footnote_dir)?;
        let contacts_dir = footnote_dir.join("contacts");
        fs::create_dir_all(&contacts_dir)?;
        let footnotes_dir = self.path.join("footnotes");
        fs::create_dir_all(&footnotes_dir)?;
        Ok(())
    }

    pub fn is_primary_device(&self) -> anyhow::Result<bool> {
        Ok(self.path.join(".footnote").join("id_key").exists())
    }

    pub fn is_created(&self) -> Result<bool> {
        Ok(self.path.join(".footnote").join("device_key").exists())
    }

    pub fn can_device_read_note(
        &self,
        device_endpoint: &iroh::PublicKey,
        note_path: &Path,
    ) -> Result<bool> {
        if self.owned_device_endpoint_to_name(device_endpoint).is_ok() {
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

    pub fn owned_device_endpoint_to_name(
        &self,
        endpoint_id: &iroh::PublicKey,
    ) -> anyhow::Result<String> {
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

    pub fn find_primary_device_by_nickname(&self, nickname: &str) -> Result<iroh::PublicKey> {
        let contacts_dir = self.path.join(".footnote").join("contacts");

        for entry in fs::read_dir(contacts_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let contact = Contact::from_file(entry.path())?;

                if contact.nickname == nickname {
                    for device in &contact.devices {
                        if let Ok(device_endpoint) =
                            device.iroh_endpoint_id.parse::<iroh::PublicKey>()
                        {
                            return Ok(device_endpoint);
                        }
                    }
                }
            }
        }

        anyhow::bail!(
            "No contact found with nickname {} or they have no devices",
            nickname
        )
    }

    pub fn contact_read(&self) -> anyhow::Result<Vec<Contact>> {
        let contacts_dir = self.path.join(".footnote").join("contacts");

        fs::read_dir(contacts_dir)?
            .filter_map(|entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => return Some(Err(e.into())),
                };

                if entry.path().extension()?.to_str()? == "json" {
                    Some(Contact::from_file(entry.path()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn contact_import(&self, nickname: &str, contact_json: &str) -> anyhow::Result<()> {
        let mut contact = Contact::from_json(contact_json)?;
        contact.verify()?; // currently called in from_json but doesn't hurt to do it here too
        contact.nickname = nickname.to_string();
        let contacts_file = self
            .path
            .join(".footnote")
            .join("contacts")
            .join(format!("{}.json", nickname));
        contact.to_file(contacts_file)?;
        Ok(())
    }

    /// return a list of devices owned by this vault
    pub fn device_read(&self) -> anyhow::Result<Vec<Device>> {
        // if user.json exists, return those
        // else: return local device
        let user_record = self.path.join(".footnote").join("user.json");

        if user_record.exists() {
            let owned_devices_record = Contact::from_file(user_record)?;
            return Ok(owned_devices_record.devices);
        }

        let (iroh_endpoint_id, device_name) = self.device_public_key()?;
        Ok([Device::new(device_name, iroh_endpoint_id.to_string())].to_vec())
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

    pub fn device_public_key(&self) -> Result<(iroh::EndpointId, String)> {
        let (secret_key, device_name) = self.device_secret_key()?;
        Ok((secret_key.public(), device_name))
    }

    pub fn device_key_update(&self, device_name: &str) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let content = fs::read_to_string(&device_key_file)?;
        let (encoded_key, _) = match content.split_once(' ') {
            Some((a, b)) => (a, b),
            None => anyhow::bail!("username not found in key"),
        };
        let device_line = format!("{} {}", encoded_key, device_name);
        fs::write(&device_key_file, device_line)?;
        Ok(())
    }

    pub fn user_read(&self) -> anyhow::Result<Option<Contact>> {
        let user_record = self.path.join(".footnote").join("user.json");

        if user_record.exists() {
            let user_record = Contact::from_file(user_record)?;
            return Ok(Some(user_record));
        }
        // probably want to grab username from id_key if it exists
        Ok(None)
    }

    pub fn user_update(&self, username: &str) -> anyhow::Result<Contact> {
        // the Vault concept and LocalUser are muddled. it's not really clear if
        // both are needed
        let local_user = LocalUser::new(&self.path)?;
        local_user.username_update(username)
    }

    pub fn note_create(&self, path: &Path, content: &str) -> anyhow::Result<()> {
        Note::create(path, content)?;
        Ok(())
    }
}

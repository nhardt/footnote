use crate::model::contact;
use crate::model::device::Device;
use crate::model::{contact::Contact, note::Note, user::LocalUser};
use anyhow::Result;
use core::fmt;
use iroh::Endpoint;
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::remove_dir_all;
use tokio::sync::mpsc::{self, Receiver};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

#[derive(Clone)]
pub struct Vault {
    pub path: PathBuf,
}

#[derive(PartialEq)]
pub enum VaultState {
    Primary,
    SecondaryJoined,
    StandAlone,
    Uninitialized,
}

impl fmt::Display for VaultState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VaultState::Primary => write!(f, "Primary"),
            VaultState::SecondaryJoined => write!(f, "Joined"),
            VaultState::StandAlone => write!(f, "Stand Alone"),
            VaultState::Uninitialized => write!(f, "Uninitialzed"),
        }
    }
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

    pub fn create_standalone(path: &Path) -> Result<Self> {
        let v = Self {
            path: path.to_path_buf(),
        };
        v.create_directory_structure()?;
        Ok(v)
    }
    /// reset device to standalone state
    pub fn transition_to_standalone(&self) -> Result<()> {
        fs::remove_file(self.path.join(".footnote").join("device_key"))?;
        fs::remove_file(self.path.join(".footnote").join("user.json"))?;
        fs::remove_file(self.path.join(".footnote").join("id_key"))?;
        self.create_directory_structure()?;
        Ok(())
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

    pub fn base_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn absolute_path_to_relative_string(&self, full_path: PathBuf) -> String {
        full_path
            .strip_prefix(self.base_path())
            .unwrap_or(&full_path)
            .to_string_lossy()
            .to_string()
    }

    pub fn relative_string_to_absolute_path(&self, relative_path: &str) -> PathBuf {
        self.base_path().join(relative_path)
    }

    pub fn relative_string_to_absolute_string(&self, relative_path: &str) -> String {
        self.base_path()
            .join(relative_path)
            .to_string_lossy()
            .to_string()
    }

    pub async fn build_endpoint(&self, alpn: &[u8]) -> Result<Endpoint> {
        let Ok((secret_key, _)) = self.device_secret_key() else {
            anyhow::bail!("could not get secret key");
        };
        let Ok(endpoint) = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![alpn.to_vec()])
            .bind()
            .await
        else {
            anyhow::bail!("could not get secret key");
        };
        Ok(endpoint)
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

        let note = Note::from_path(note_path, false)?;

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

        if !contacts_dir.exists() {
            return Ok(Vec::new());
        }

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

    pub fn contact_update(&self, nickname: &str, new_contact: &mut Contact) -> anyhow::Result<()> {
        let contact_file_path = self.path.join(".footnote").join("contacts").join(nickname);
        let current_contact = Contact::from_file(&contact_file_path)?;
        current_contact.verify()?;
        new_contact.verify()?;

        if let Err(e) = new_contact.is_valid_successor_of(&current_contact) {
            tracing::error!("failed successor check: {}", e);
            anyhow::bail!("received invalid user record update");
        }

        new_contact.nickname = nickname.to_string();
        new_contact.to_file(contact_file_path)?;
        Ok(())
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

        let (iroh_endpoint_id, device_name) = match self.device_public_key() {
            Ok(r) => r,
            Err(_) => {
                return Ok(Vec::new());
            }
        };
        Ok([Device::new(device_name, iroh_endpoint_id.to_string())].to_vec())
    }

    pub fn device_delete(&self, iroh_endpoint: &str) -> anyhow::Result<()> {
        let local_user = LocalUser::new(&self.path)?;
        local_user.device_delete_from_contact_record(&iroh_endpoint)?;
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

    pub fn user_write(&self, user: &Contact) -> anyhow::Result<()> {
        let user_record_path = self.path.join(".footnote").join("user.json");
        user.to_file(user_record_path)?;
        Ok(())
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

    pub fn doctor(&self, fix: bool) -> Result<Vec<(String, String)>> {
        let mut ret = Vec::new();
        let mut uuids = HashMap::new();
        let mut needs_new_uuid = Vec::new();
        let mut needs_frontmatter = Vec::new();

        let is_hidden = |e: &DirEntry| {
            e.file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        };

        for entry in WalkDir::new(self.base_path())
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.') {
                    continue;
                }
            }

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let Ok(_) = Note::from_path(path, true) else {
                ret.push((
                    path.to_string_lossy().to_string(),
                    "cannot coerce to note".to_string(),
                ));
                continue;
            };

            let Ok(note) = Note::from_path(path, false) else {
                ret.push((
                    path.to_string_lossy().to_string(),
                    "does not parse as note".to_string(),
                ));
                needs_frontmatter.push(path.to_path_buf());
                continue;
            };

            if note.frontmatter.uuid.is_nil() {
                ret.push((
                    path.to_string_lossy().to_string(),
                    "has a nil uuid".to_string(),
                ));
                continue;
            }

            match uuids.entry(note.frontmatter.uuid) {
                Entry::Vacant(uuid_entry) => {
                    uuid_entry.insert((note.frontmatter.uuid, path.to_string_lossy().to_string()));
                }
                Entry::Occupied(uuid_entry) => {
                    ret.push((
                        path.to_string_lossy().to_string(),
                        format!(
                            "{} duplicates {}",
                            path.to_string_lossy(),
                            uuid_entry.get().1
                        ),
                    ));
                    needs_new_uuid.push(path.to_path_buf());
                    continue;
                }
            }
        }

        if fix {
            for rewrite in needs_new_uuid {
                if let Ok(mut note) = Note::from_path(&rewrite, false) {
                    note.frontmatter.uuid = Uuid::new_v4();
                    if let Err(_) = note.to_file(&rewrite) {
                        ret.push((
                            rewrite.to_string_lossy().to_string(),
                            format!(
                                "{} could not rewrite",
                                rewrite.to_string_lossy().to_string()
                            ),
                        ));
                    }
                };
            }

            for note_without_metdata in needs_frontmatter {
                if let Ok(mut note) = Note::from_path(&note_without_metdata, true) {
                    if let Err(_) = note.to_file(&note_without_metdata) {
                        ret.push((
                            note_without_metdata.to_string_lossy().to_string(),
                            format!(
                                "{} could not add metadata",
                                note_without_metdata.to_string_lossy().to_string()
                            ),
                        ));
                    }
                };
            }
        }

        Ok(ret)
    }
}

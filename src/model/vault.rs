use crate::util::crypto;
use anyhow::Result;
use iroh::{Endpoint, SecretKey};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Vault {
    path: PathBuf,
}

impl Vault {
    pub fn create_primary(path: PathBuf, username: &str, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        v.create_identity_signing_key()?;
        Ok(v)
    }

    pub fn create_secondary(path: PathBuf) -> Result<Self> {
        v.create_directory_structure()?;
        Ok(Self { path })
    }

    pub fn join(&self, connection_string: &str, device_name: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// inside a footnote vault:
    ///
    /// .footnote/
    ///    contacts/          : your imported contacts are stored here as json records
    ///    master_identity    : a private key used to identify you across all your devices
    ///    this_device        : a plain text file that indicates this device in contact.json
    ///    contact.json       : your device record, signed with your public key
    /// footnotes/            : notes shared with you by other people
    /// home.md               : default note
    fn create_directory_structure(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let contacts_dir = footnotes_dir.join("contacts");
        let trusted_sources_dir = self.path.join("footnotes");

        fs::create_dir_all(&footnotes_dir)?;
        fs::create_dir_all(&contacts_dir)?;
        fs::create_dir_all(&trusted_sources_dir)?;

        Ok(())
    }

    /// identity signing key is the master key for the vault. it's generated and stored
    /// on the primary device. this idenity signing key is used to sign the contact record,
    /// and represents a user's stable identity.
    fn create_identity_signing_key(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let identity_signing_key_file = footnotes_dir.join("identity_signing_key");
        let (signing_key, _verifying_key) = crypto::generate_identity_keypair();
        fs::write(
            &identity_signing_key_file,
            crypto::signing_key_to_hex(&signing_key),
        )?;
        Ok(())
    }

    /// the device signing key is generated and stored local to each device. it
    /// is used in establishing verified connections between devices via iroh.
    /// devices are stored in the contact record. to add a device to the contact
    /// record, a secondary device contacts the primary device with a join code,
    /// the primary device mints a new contact record. the contact record
    /// contains public key information.
    fn create_device_signing_key(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let device_signing_key_file = footnotes_dir.join("device_signing_key");
        let iroh_secret_key = iroh::SecretKey::generate(&mut rand::rng());
        fs::write(&device_signing_key_file, iroh_secret_key.to_bytes())?;
        Ok(())
    }
}

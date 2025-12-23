use crate::model::contact::Contact;
use crate::model::device::Device;
use crate::util::crypto;
use anyhow::Result;
use ed25519_dalek::{SigningKey, VerifyingKey};
use iroh::{Endpoint, SecretKey};
use rand_core::OsRng;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Vault {
    path: PathBuf,
}

impl Vault {
    /// called on the first device when creating a new vault
    pub fn create_primary(path: PathBuf, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        v.create_id_key()?;
        v.create_device_key()?;
        Ok(v)
    }

    /// called on non-primary device to put vault into state where it's ready to
    /// join
    pub fn create_secondary(path: PathBuf, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        v.create_device_key()?;
        Ok(v)
    }

    pub fn join(&self, connection_string: &str, device_name: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// inside a footnote vault:
    ///
    /// .footnote/
    ///    id_key               : private key that signs device record, primary only
    ///    device_key           : private key specific to this device
    fn create_directory_structure(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        fs::create_dir_all(&footnotes_dir)?;
        Ok(())
    }

    /// identity signing key is the master key for the vault. it's generated and
    /// stored on the primary device. this idenity signing key is used to sign
    /// the contact record, and represents a user's stable identity.
    fn create_id_key(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let id_key_file = footnotes_dir.join("id_key");
        let mut csprng = OsRng;
        let id_key = SigningKey::generate(&mut csprng);
        let id_key_hex = hex::encode(id_key.to_bytes());
        fs::write(&id_key_file, id_key_hex)?;
        Ok(())
    }

    /// the device signing key is generated and stored local to each device. it
    /// is used in establishing verified connections between devices via iroh.
    fn create_device_key(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let device_key_file = footnotes_dir.join("device_key");
        let device_key = iroh::SecretKey::generate(&mut rand::rng());
        fs::write(&device_key_file, device_key.to_bytes())?;
        Ok(())
    }
}

use crate::model::contact::Contact;
use crate::model::device::Device;
use crate::util::crypto;
use anyhow::Result;
use iroh::{Endpoint, SecretKey};
use std::fs;
use std::path::{Path, PathBuf};
use ed25519_dalek::{SigningKey, VerifyingKey};

pub struct Vault {
    path: PathBuf,
}

impl Vault {
    pub fn create_primary(path: PathBuf, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        v.create_identity_signing_key()?;
        let this_device = Device{
            name: device_name,
            iroh_endpoint_id: v.create_device_signing_key()?,
        };
        v.create_contact_record(this_device)?;
        Ok(v)
    }

    pub fn create_secondary(path: PathBuf, device_name: &str) -> Result<Self> {
        let v = Self { path };
        v.create_directory_structure()?;
        v.create_device_signing_key()?;
        Ok(v)
    }

    pub fn join(&self, connection_string: &str, device_name: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// inside a footnote vault:
    ///
    /// .footnote/
    ///    contacts/            : your imported contacts are stored here as json records
    ///    identity_signing_key : private key that signs device record, primary only
    ///    device_signing_key   : private key specific to this device
    ///    contact.json         : device list, contact card. managed on primary.
    /// footnotes/              : notes shared with you by other people
    /// home.md                 : default note
    fn create_directory_structure(&self) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let contacts_dir = footnotes_dir.join("contacts");
        fs::create_dir_all(&footnotes_dir)?;
        fs::create_dir_all(&contacts_dir)?;

        // potentially we should only create this after importing the first
        // contact record
        let trusted_sources_dir = self.path.join("footnotes");
        fs::create_dir_all(&trusted_sources_dir)?;

        Ok(())
    }

    /// identity signing key is the master key for the vault. it's generated and stored
    /// on the primary device. this idenity signing key is used to sign the contact record,
    /// and represents a user's stable identity.
    fn create_identity_signing_key(&self) -> anyhow::Result<VerifyingKey> {
        let footnotes_dir = self.path.join(".footnote");
        let identity_signing_key_file = footnotes_dir.join("identity_signing_key");
        let (signing_key, verifying_key) = crypto::generate_identity_keypair();
        fs::write(
            &identity_signing_key_file,
            crypto::signing_key_to_hex(&signing_key),
        )?;
        Ok(verifying_key)
    }

    /// the device signing key is generated and stored local to each device. it
    /// is used in establishing verified connections between devices via iroh.
    /// devices are stored in the contact record. to add a device to the contact
    /// record, a secondary device contacts the primary device with a join code,
    /// the primary device mints a new contact record. the contact record
    /// contains public key information.
    fn create_device_signing_key(&self) -> anyhow::Result<iroh::PublicKey> {
        let footnotes_dir = self.path.join(".footnote");
        let device_signing_key_file = footnotes_dir.join("device_signing_key");
        let iroh_secret_key = iroh::SecretKey::generate(&mut rand::rng());
        fs::write(&device_signing_key_file, iroh_secret_key.to_bytes())?;
        Ok(iroh_secret_key.public())
    }

    /// verified RMW semantics:
    /// - verify: no record or old record verifies
    /// - modify: add or remove devices
    /// - write: update timestamp, sign, write
    fn create_contact_record(&self, this_device: Device) -> anyhow::Result<()> {
        let footnotes_dir = self.path.join(".footnote");
        let self_contact_file = footnotes_dir.join("contact.json");
        let record = if self_contact_file.exists() {
            Contact::from_file(self_contact_file);
        } else {
            // by default a user won't have a username or nickname. when they
            // export their contact record they can add their username.
            // nickname/petname doesn't actually make sense right now. since
            // the user getting this record can't modify it, they can't have
            // a nickname. maybe nickname can be outside of the verification.
            Contact {
                username: "".to_string(),
                nickname: "".to_string(),
                identity_verifying_key:,
                devices: [this_device].to_vec(),
            };
        };

        Ok(())
    }
}

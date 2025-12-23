use crate::model::device::Device;
use crate::model::lamport_timestamp::LamportTimestamp;
use crate::util::crypto;
use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub nickname: String,
    pub username: String,
    pub id_public_key: String,
    pub devices: Vec<Device>,
    pub updated_at: LamportTimestamp,
    #[serde(default)]
    signature: String,
}

#[derive(Serialize)]
pub struct SignableContact<'a> {
    username: &'a str,
    id_public_key: &'a str,
    devices: &'a [Device],
    updated_at: LamportTimestamp,
}

impl Contact {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read contact file: {}", path.as_ref().display()))?;
        Self::from_json(&content)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let contact: Contact =
            serde_json::from_str(json).context("Failed to parse contact JSON")?;
        contact.verify()?;
        Ok(contact)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("Failed to serialize contact")
    }

    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize contact")
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = self.to_json_pretty()?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn new_local_user_record(
        username: &str,
        id_public_key: &str,
        primary_device: Device,
        id_signing_key: &SigningKey,
    ) -> Result<Contact> {
        let mut c = Contact {
            nickname: "".to_string(),
            username: username.to_string(),
            id_public_key: id_public_key.to_string(),
            devices: [primary_device].to_vec(),
            updated_at: LamportTimestamp(0),
            signature: "".to_string(),
        };
        c.sign(id_signing_key)?;
        Ok(c)
    }

    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<()> {
        let signable = SignableContact {
            username: &self.username,
            id_public_key: &self.id_public_key,
            devices: &self.devices,
            updated_at: self.updated_at,
        };
        let message = serde_json::to_string(&signable)?;
        let signature = signing_key.sign(message.as_bytes());
        self.signature = hex::encode(signature.to_bytes());
        Ok(())
    }

    pub fn verify(&self) -> Result<()> {
        let verifying_key = crypto::verifying_key_from_hex(&self.id_public_key)?;

        let signable = SignableContact {
            username: &self.username,
            id_public_key: &self.id_public_key,
            devices: &self.devices,
            updated_at: self.updated_at,
        };

        let message = serde_json::to_string(&signable)?;

        let signature_bytes = match hex::decode(&self.signature) {
            Ok(bytes) => bytes,
            Err(_) => anyhow::bail!("no signature"),
        };

        let signature = match Signature::from_slice(&signature_bytes) {
            Ok(sig) => sig,
            Err(_) => anyhow::bail!("could no create signature"),
        };

        match verifying_key.verify(message.as_bytes(), &signature) {
            Ok(_) => Ok(()),
            Err(_) => anyhow::bail!("contact record not verified!"),
        }
    }

    pub fn validate_successor(&self, previous: &Contact) -> Result<()> {
        if self.id_public_key != previous.id_public_key {
            anyhow::bail!("successor is older than current")
        }
        self.verify()?;
        previous.verify()?;

        if self.updated_at <= previous.updated_at {
            anyhow::bail!("successor is older than current")
        }

        Ok(())
    }
}

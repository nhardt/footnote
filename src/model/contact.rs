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

        if !contact.verify()? {
            anyhow::bail!("Contact signature verification failed");
        }

        Ok(contact)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("Failed to serialize contact")
    }

    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize contact")
    }

    pub fn verify(&self) -> Result<bool> {
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
            Err(_) => return Ok(false),
        };

        let signature = match Signature::from_slice(&signature_bytes) {
            Ok(sig) => sig,
            Err(_) => return Ok(false),
        };

        match verifying_key.verify(message.as_bytes(), &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn is_valid_successor(&self, previous: &Contact) -> Result<bool> {
        if self.id_public_key != previous.id_public_key {
            return Ok(false);
        }

        if !self.verify()? || !previous.verify()? {
            return Ok(false);
        }

        if self.updated_at <= previous.updated_at {
            return Ok(false);
        }

        Ok(true)
    }
}

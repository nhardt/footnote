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
    pub username: String,
    pub nickname: String,
    pub master_public_key: String,
    pub devices: Vec<Device>,
    pub updated_at: LamportTimestamp,
    #[serde(default)]
    signature: String,
}

impl Contact {
    pub fn verify(&self) -> Result<bool> {
        let verifying_key = crypto::verifying_key_from_hex(&self.master_public_key)?;

        let mut unsigned = self.clone();
        unsigned.signature = String::new();

        let message = serde_json::to_string(&unsigned)?;

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
        // Must have same master public key
        if self.master_public_key != previous.master_public_key {
            return Ok(false);
        }

        // Both must verify
        if !self.verify()? || !previous.verify()? {
            return Ok(false);
        }

        // Must have newer timestamp
        if self.updated_at <= previous.updated_at {
            return Ok(false);
        }

        Ok(true)
    }

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

    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<()> {
        // Clear existing signature
        self.signature = String::new();

        // Serialize unsigned record
        let message = serde_json::to_string(self)?;

        // Sign
        let signature = signing_key.sign(message.as_bytes());
        self.signature = hex::encode(signature.to_bytes());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_signing_key() -> SigningKey {
        let (signing_key, _) = crypto::generate_identity_keypair();
        signing_key
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            master_public_key: crypto::verifying_key_to_hex(&verifying_key),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();
        assert!(contact.verify().unwrap());
    }

    #[test]
    fn test_verify_fails_with_wrong_signature() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            master_public_key: crypto::verifying_key_to_hex(&verifying_key),
            devices: vec![],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();

        // Tamper with the signature
        contact.signature = "0000".to_string();
        assert!(!contact.verify().unwrap());
    }

    #[test]
    fn test_is_valid_successor() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();
        let master_key = crypto::verifying_key_to_hex(&verifying_key);

        let mut contact_v1 = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            master_public_key: master_key.clone(),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };
        contact_v1.sign(&signing_key).unwrap();

        let mut contact_v2 = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            master_public_key: master_key.clone(),
            devices: vec![
                Device::new("laptop".to_string(), "abc123".to_string()),
                Device::new("phone".to_string(), "def456".to_string()),
            ],
            updated_at: LamportTimestamp(2000),
            signature: String::new(),
        };
        contact_v2.sign(&signing_key).unwrap();

        assert!(contact_v2.is_valid_successor(&contact_v1).unwrap());
        assert!(!contact_v1.is_valid_successor(&contact_v2).unwrap()); // older
    }

    #[test]
    fn test_json_round_trip() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "Alice W.".to_string(),
            master_public_key: crypto::verifying_key_to_hex(&verifying_key),
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

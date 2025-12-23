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
struct SignableContact<'a> {
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

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("Failed to serialize contact")
    }

    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize contact")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    fn create_test_signing_key() -> SigningKey {
        let mut csprng = OsRng;
        SigningKey::generate(&mut csprng)
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();
        assert!(contact.verify().unwrap());
    }

    #[test]
    fn test_nickname_not_verified() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "Alice W.".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();

        // Changing nickname should not break verification
        contact.nickname = "Different Name".to_string();
        assert!(contact.verify().unwrap());
    }

    #[test]
    fn test_verify_fails_with_wrong_signature() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            devices: vec![],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();

        contact.signature = "0000".to_string();
        assert!(!contact.verify().unwrap());
    }

    #[test]
    fn test_is_valid_successor() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();
        let master_key = hex::encode(verifying_key.to_bytes());

        let mut contact_v1 = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };
        contact_v1.sign(&signing_key).unwrap();

        let mut contact_v2 = Contact {
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            devices: vec![
                Device::new("laptop".to_string(), "abc123".to_string()),
                Device::new("phone".to_string(), "def456".to_string()),
            ],
            updated_at: LamportTimestamp(2000),
            signature: String::new(),
        };
        contact_v2.sign(&signing_key).unwrap();

        assert!(contact_v2.is_valid_successor(&contact_v1).unwrap());
        assert!(!contact_v1.is_valid_successor(&contact_v2).unwrap());
    }

    #[test]
    fn test_json_round_trip() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            username: "alice".to_string(),
            nickname: "Alice W.".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
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

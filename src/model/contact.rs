use crate::model::device::Device;
use crate::util::crypto;
use crate::util::lamport_timestamp::LamportTimestamp;
use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    pub format_version: u32,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub nickname: String,
    pub username: String,
    pub id_public_key: String,
    /// iroh endpoint id of device leader
    pub device_leader: String,
    pub devices: Vec<Device>,
    pub updated_at: LamportTimestamp,
    #[serde(default)]
    signature: String,
}

#[derive(Serialize)]
pub struct SignableContact<'a> {
    format_version: u32,
    username: &'a str,
    id_public_key: &'a str,
    device_leader: &'a str,
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
            format_version: FORMAT_VERSION,
            nickname: "".to_string(),
            username: username.to_string(),
            id_public_key: id_public_key.to_string(),
            device_leader: primary_device.iroh_endpoint_id.to_string(),
            devices: [primary_device].to_vec(),
            updated_at: LamportTimestamp::new(None),
            signature: "".to_string(),
        };
        c.sign(id_signing_key)?;
        Ok(c)
    }

    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<()> {
        self.updated_at = LamportTimestamp::new(Some(self.updated_at));

        let signable = SignableContact {
            format_version: self.format_version,
            username: &self.username,
            id_public_key: &self.id_public_key,
            device_leader: &self.device_leader,
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
            format_version: self.format_version,
            username: &self.username,
            id_public_key: &self.id_public_key,
            device_leader: &self.device_leader,
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

    pub fn is_valid_successor_of(&self, previous: &Contact) -> Result<()> {
        //TODO: support case where successor is a leadership change
        anyhow::ensure!(
            self.id_public_key == previous.id_public_key,
            "cannot update user record, public key id mismatch"
        );
        anyhow::ensure!(
            self.updated_at > previous.updated_at
                || (self.updated_at == previous.updated_at
                    && self.signature == previous.signature
                    && !previous.signature.is_empty()),
            "Successor is not newer or is same record with matching signature"
        );
        self.verify()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
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
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            device_leader: "abc123".to_string(),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();
        contact.verify().unwrap();
    }

    #[test]
    fn test_nickname_not_verified() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "Alice W.".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            device_leader: "abc123".to_string(),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();

        // Changing nickname should not break verification
        contact.nickname = "Different Name".to_string();
        contact.verify().unwrap();
    }

    #[test]
    fn test_verify_fails_with_wrong_signature() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            device_leader: "".to_string(),
            devices: vec![],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };

        contact.sign(&signing_key).unwrap();

        contact.signature = "0000".to_string();

        match contact.verify() {
            Ok(_) => panic!("did not detect signature mismatch"),
            Err(_) => println!("detected signature mismatch"),
        }
    }

    #[test]
    fn test_is_valid_successor() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();
        let master_key = hex::encode(verifying_key.to_bytes());

        let mut contact_v1 = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            device_leader: "abc123".to_string(),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp::new(None),
            signature: String::new(),
        };
        contact_v1.sign(&signing_key).unwrap();

        let mut contact_v2 = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            device_leader: "abc123".to_string(),
            devices: vec![
                Device::new("laptop".to_string(), "abc123".to_string()),
                Device::new("phone".to_string(), "def456".to_string()),
            ],
            updated_at: LamportTimestamp::new(Some(contact_v1.updated_at)),
            signature: String::new(),
        };
        contact_v2.sign(&signing_key).unwrap();

        contact_v2.is_valid_successor_of(&contact_v1).unwrap();

        match contact_v1.is_valid_successor_of(&contact_v2) {
            Ok(_) => panic!("allowed invalid successor to contact_v1"),
            Err(_) => println!("detected invalid successor to contact_v1"),
        }
    }

    #[test]
    fn test_json_round_trip() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let mut contact = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "Alice W.".to_string(),
            id_public_key: hex::encode(verifying_key.to_bytes()),
            device_leader: "abc123".to_string(),
            devices: vec![Device::new("laptop".to_string(), "abc123".to_string())],
            updated_at: LamportTimestamp(1000),
            signature: String::new(),
        };
        contact.sign(&signing_key).unwrap();

        let json = contact.to_json().unwrap();
        let loaded = Contact::from_json(&json).unwrap();

        assert_eq!(loaded.username, contact.username);
        assert_eq!(loaded.devices.len(), contact.devices.len());
        contact.verify().unwrap();
    }

    #[test]
    fn test_leader_transfer_valid_chain() {
        let signing_key_a = create_test_signing_key();
        let verifying_key_a = signing_key_a.verifying_key();
        let master_key_a = hex::encode(verifying_key_a.to_bytes());

        let signing_key_d = create_test_signing_key();
        let verifying_key_d = signing_key_d.verifying_key();
        let master_key_d = hex::encode(verifying_key_d.to_bytes());

        // Initial record: A is leader, signed by A
        let mut contact_v1 = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key_a.clone(),
            device_leader: "node_a".to_string(),
            devices: vec![
                Device::new("desktop".to_string(), "node_a".to_string()),
                Device::new("laptop".to_string(), "node_b".to_string()),
                Device::new("phone".to_string(), "node_d".to_string()),
            ],
            updated_at: LamportTimestamp::new(None),
            signature: String::new(),
        };
        contact_v1.sign(&signing_key_a).unwrap();

        // Transfer record: D is new leader, still signed by A
        let mut transfer_record = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key_a.clone(),
            device_leader: "node_d".to_string(),
            devices: contact_v1.devices.clone(),
            updated_at: LamportTimestamp::new(Some(contact_v1.updated_at)),
            signature: String::new(),
        };
        transfer_record.sign(&signing_key_a).unwrap();

        // Verify transfer record is valid successor
        transfer_record.is_valid_successor_of(&contact_v1).unwrap();

        // Takeover record: D is leader, signed by D's new key
        let mut takeover_record = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key_d.clone(),
            device_leader: "node_d".to_string(),
            devices: transfer_record.devices.clone(),
            updated_at: LamportTimestamp::new(Some(transfer_record.updated_at)),
            signature: String::new(),
        };
        takeover_record.sign(&signing_key_d).unwrap();

        // Verify takeover record is valid successor to transfer record
        takeover_record
            .is_valid_successor_of(&transfer_record)
            .unwrap();
    }

    #[test]
    fn test_leader_transfer_missing_transfer_record_fails() {
        let signing_key_a = create_test_signing_key();
        let verifying_key_a = signing_key_a.verifying_key();
        let master_key_a = hex::encode(verifying_key_a.to_bytes());

        let signing_key_d = create_test_signing_key();
        let verifying_key_d = signing_key_d.verifying_key();
        let master_key_d = hex::encode(verifying_key_d.to_bytes());

        // Initial record: A is leader
        let mut contact_v1 = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key_a.clone(),
            device_leader: "node_a".to_string(),
            devices: vec![
                Device::new("desktop".to_string(), "node_a".to_string()),
                Device::new("phone".to_string(), "node_d".to_string()),
            ],
            updated_at: LamportTimestamp::new(None),
            signature: String::new(),
        };
        contact_v1.sign(&signing_key_a).unwrap();

        // Attacker tries to skip transfer record and claim leadership with new key
        let mut malicious_takeover = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key_d.clone(),
            device_leader: "node_d".to_string(),
            devices: contact_v1.devices.clone(),
            updated_at: LamportTimestamp::new(Some(contact_v1.updated_at)),
            signature: String::new(),
        };
        malicious_takeover.sign(&signing_key_d).unwrap();

        // Should fail - can't validate with new key without transfer record
        match malicious_takeover.is_valid_successor_of(&contact_v1) {
            Ok(_) => panic!("allowed takeover without transfer record"),
            Err(_) => println!("correctly rejected takeover without transfer record"),
        }
    }

    #[test]
    fn test_device_leader_change_without_key_rotation() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();
        let master_key = hex::encode(verifying_key.to_bytes());

        // Initial: device A is leader
        let mut contact_v1 = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            device_leader: "node_a".to_string(),
            devices: vec![
                Device::new("desktop".to_string(), "node_a".to_string()),
                Device::new("laptop".to_string(), "node_b".to_string()),
            ],
            updated_at: LamportTimestamp::new(None),
            signature: String::new(),
        };
        contact_v1.sign(&signing_key).unwrap();

        // Just changing which device is leader, same signing key
        let mut contact_v2 = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            device_leader: "node_b".to_string(),
            devices: contact_v1.devices.clone(),
            updated_at: LamportTimestamp::new(Some(contact_v1.updated_at)),
            signature: String::new(),
        };
        contact_v2.sign(&signing_key).unwrap();

        // Should be valid - same key, higher timestamp
        contact_v2.is_valid_successor_of(&contact_v1).unwrap();
    }

    #[test]
    fn test_cannot_change_device_leader_in_signature() {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();
        let master_key = hex::encode(verifying_key.to_bytes());

        let mut contact = Contact {
            format_version: FORMAT_VERSION,
            username: "alice".to_string(),
            nickname: "".to_string(),
            id_public_key: master_key.clone(),
            device_leader: "node_a".to_string(),
            devices: vec![Device::new("desktop".to_string(), "node_a".to_string())],
            updated_at: LamportTimestamp::new(None),
            signature: String::new(),
        };
        contact.sign(&signing_key).unwrap();

        // Attacker changes device_leader after signing
        contact.device_leader = "node_evil".to_string();

        // Should fail validation
        match contact.verify() {
            Ok(_) => panic!("allowed tampering with device_leader"),
            Err(_) => println!("correctly detected tampered device_leader"),
        }
    }
}

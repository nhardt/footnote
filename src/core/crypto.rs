use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

/// Generate a new Ed25519 identity keypair
pub fn generate_identity_keypair() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Device record to be signed
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub device_name: String,
    pub iroh_endpoint_id: String,
    pub authorized_by: String, // hex-encoded verifying key
    pub timestamp: String,
}

/// Sign a device record with the master signing key
pub fn sign_device_record(
    device_name: &str,
    iroh_endpoint_id: &str,
    master_key: &SigningKey,
    timestamp: &str,
) -> anyhow::Result<String> {
    let verifying_key = master_key.verifying_key();
    let authorized_by = hex::encode(verifying_key.to_bytes());

    let record = DeviceRecord {
        device_name: device_name.to_string(),
        iroh_endpoint_id: iroh_endpoint_id.to_string(),
        authorized_by,
        timestamp: timestamp.to_string(),
    };

    // Serialize the record to create a canonical representation
    let message = serde_yaml::to_string(&record)?;

    // Sign the message
    let signature = master_key.sign(message.as_bytes());

    Ok(hex::encode(signature.to_bytes()))
}

/// Verify a device record signature
pub fn verify_device_signature(
    device_name: &str,
    iroh_endpoint_id: &str,
    authorized_by: &str,
    timestamp: &str,
    signature_hex: &str,
) -> anyhow::Result<bool> {
    // Reconstruct the device record
    let record = DeviceRecord {
        device_name: device_name.to_string(),
        iroh_endpoint_id: iroh_endpoint_id.to_string(),
        authorized_by: authorized_by.to_string(),
        timestamp: timestamp.to_string(),
    };

    // Serialize to get the same canonical representation
    let message = serde_yaml::to_string(&record)?;

    // Decode the signature
    let signature_bytes = hex::decode(signature_hex)?;
    let signature = Signature::from_slice(&signature_bytes)?;

    // Decode the verifying key
    let key_bytes = hex::decode(authorized_by)?;
    let verifying_key = VerifyingKey::from_bytes(
        &key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?,
    )?;

    // Verify the signature
    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Convert a signing key to hex string for storage
pub fn signing_key_to_hex(key: &SigningKey) -> String {
    hex::encode(key.to_bytes())
}

/// Convert a verifying key to hex string for storage
pub fn verifying_key_to_hex(key: &VerifyingKey) -> String {
    hex::encode(key.to_bytes())
}

/// Load a signing key from hex string
pub fn signing_key_from_hex(hex_str: &str) -> anyhow::Result<SigningKey> {
    let bytes = hex::decode(hex_str)?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

/// Load a verifying key from hex string
pub fn verifying_key_from_hex(hex_str: &str) -> anyhow::Result<VerifyingKey> {
    let bytes = hex::decode(hex_str)?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    Ok(VerifyingKey::from_bytes(&key_bytes)?)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactDevice {
    pub device_name: String,
    pub iroh_endpoint_id: String,
    pub added_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactRecord {
    pub username: String,
    pub nickname: String,
    pub master_public_key: String,
    pub devices: Vec<ContactDevice>,
    pub updated_at: String,
    #[serde(default)]
    pub signature: String,
}

pub fn sign_contact_record(
    record: &ContactRecord,
    master_key: &SigningKey,
) -> anyhow::Result<String> {
    let mut unsigned_record = record.clone();
    unsigned_record.signature = String::new();

    let message = serde_json::to_string(&unsigned_record)?;
    let signature = master_key.sign(message.as_bytes());

    Ok(hex::encode(signature.to_bytes()))
}

pub fn verify_contact_signature(record: &ContactRecord) -> anyhow::Result<bool> {
    let verifying_key = verifying_key_from_hex(&record.master_public_key)?;

    let mut unsigned_record = record.clone();
    unsigned_record.signature = String::new();

    let message = serde_json::to_string(&unsigned_record)?;

    let signature_bytes = hex::decode(&record.signature)?;
    let signature = Signature::from_slice(&signature_bytes)?;

    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

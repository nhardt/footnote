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

pub fn signing_key_to_hex(key: &SigningKey) -> String {
    hex::encode(key.to_bytes())
}

pub fn verifying_key_to_hex(key: &VerifyingKey) -> String {
    hex::encode(key.to_bytes())
}

pub fn signing_key_from_hex(hex_str: &str) -> anyhow::Result<SigningKey> {
    let bytes = hex::decode(hex_str)?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

pub fn verifying_key_from_hex(hex_str: &str) -> anyhow::Result<VerifyingKey> {
    let bytes = hex::decode(hex_str)?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    Ok(VerifyingKey::from_bytes(&key_bytes)?)
}

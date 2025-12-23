use ed25519_dalek::{SigningKey, VerifyingKey};

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

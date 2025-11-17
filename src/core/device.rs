/// Create a new device for a user
pub async fn create(user_name: &str, device_name: &str) -> anyhow::Result<()> {
    println!(
        "TODO: device::create({}, {})",
        user_name, device_name
    );
    Ok(())
}

/// Delete a device
pub async fn delete(user_name: &str, device_name: &str) -> anyhow::Result<()> {
    println!(
        "TODO: device::delete({}, {})",
        user_name, device_name
    );
    Ok(())
}

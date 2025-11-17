/// Listen for incoming mirror connections
pub async fn listen() -> anyhow::Result<()> {
    println!("TODO: mirror::listen()");
    Ok(())
}

/// Push mirror data
///
/// # Arguments
/// * `user` - Optional user name to push to
/// * `device` - Optional device name (requires user)
pub async fn push(user: Option<&str>, device: Option<&str>) -> anyhow::Result<()> {
    match (user, device) {
        (None, None) => {
            println!("TODO: mirror::push() - push everything");
        }
        (Some(user_name), None) => {
            println!("TODO: mirror::push(user: {})", user_name);
        }
        (Some(user_name), Some(device_name)) => {
            println!(
                "TODO: mirror::push(user: {}, device: {})",
                user_name, device_name
            );
        }
        (None, Some(_)) => {
            anyhow::bail!("device requires user to be specified");
        }
    }
    Ok(())
}

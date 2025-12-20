use crate::core::crypto;
use serde::Serialize;
use std::fs;

#[derive(Debug, Serialize)]
struct Device {
    name: String,
    endpoint_id: String,
    authorized_by: String,
    timestamp: String,
    signature_valid: bool,
}

#[derive(Debug, Serialize)]
struct User {
    name: String,
    master_public_key: Option<String>,
    nickname: Option<String>,
    devices: Vec<Device>,
}

#[derive(Debug, Serialize)]
struct UsersOutput {
    users: Vec<User>,
}

/// Create a new user
pub async fn create(user_name: &str) -> anyhow::Result<()> {
    println!("TODO: user::create({})", user_name);
    Ok(())
}

/// Delete a user
pub async fn delete(user_name: &str) -> anyhow::Result<()> {
    println!("TODO: user::delete({})", user_name);
    Ok(())
}

/// Export a user's contact information
pub async fn export(vault_path: &std::path::Path, user_name: &str) -> anyhow::Result<()> {
    if user_name == "me" {
        return export_me(vault_path).await;
    } else {
        return export_trusted_user(vault_path, user_name).await;
    }
}

/// Export "me" user's contact information
async fn export_me(vault_path: &std::path::Path) -> anyhow::Result<()> {
    println!("{}", export_me_json_pretty(vault_path).await?);
    Ok(())
}

pub async fn export_me_json_pretty(vault_path: &std::path::Path) -> anyhow::Result<String> {
    let contact_path = vault_path.join(".footnotes").join("contact.json");

    if !contact_path.exists() {
        anyhow::bail!("Contact record not found. Run 'footnote init' first.");
    }

    let content = fs::read_to_string(&contact_path)?;
    let contact_record: crypto::ContactRecord = serde_json::from_str(&content)?;

    if !crypto::verify_contact_signature(&contact_record)? {
        anyhow::bail!("Contact record signature verification failed");
    }

    Ok(serde_json::to_string_pretty(&contact_record)?)
}

/// Export a trusted user's contact information
async fn export_trusted_user(vault_path: &std::path::Path, petname: &str) -> anyhow::Result<()> {
    let contact_path = vault_path
        .join(".footnotes")
        .join("contacts")
        .join(format!("{}.json", petname));

    if !contact_path.exists() {
        anyhow::bail!("Trusted user '{}' not found", petname);
    }

    let content = fs::read_to_string(&contact_path)?;
    let contact_record: crypto::ContactRecord = serde_json::from_str(&content)?;

    if !crypto::verify_contact_signature(&contact_record)? {
        anyhow::bail!(
            "Trusted user '{}' contact signature verification failed",
            petname
        );
    }

    println!("{}", serde_json::to_string_pretty(&contact_record)?);

    Ok(())
}

/// Import a user's contact information from a JSON string
pub async fn import_from_string(
    vault_path: &std::path::Path,
    content: &str,
    petname: &str,
) -> anyhow::Result<()> {
    let new_contact: crypto::ContactRecord = serde_json::from_str(content)?;

    if !crypto::verify_contact_signature(&new_contact)? {
        anyhow::bail!("Contact signature verification failed. Import aborted.");
    }

    eprintln!(
        "Contact signature verified ({} devices)",
        new_contact.devices.len()
    );

    let contact_path = vault_path
        .join(".footnotes")
        .join("contacts")
        .join(format!("{}.json", petname));

    if contact_path.exists() {
        let existing_content = fs::read_to_string(&contact_path)?;
        let existing_contact: crypto::ContactRecord = serde_json::from_str(&existing_content)?;

        let new_timestamp = chrono::DateTime::parse_from_rfc3339(&new_contact.updated_at)?;
        let existing_timestamp =
            chrono::DateTime::parse_from_rfc3339(&existing_contact.updated_at)?;

        if new_timestamp <= existing_timestamp {
            anyhow::bail!(
                "Trusted user '{}' already has a more recent contact record (existing: {}, new: {}). Skipping update.",
                petname,
                existing_contact.updated_at,
                new_contact.updated_at
            );
        }

        eprintln!(
            "Updating trusted user '{}' (old: {}, new: {})",
            petname, existing_contact.updated_at, new_contact.updated_at
        );
    }

    // Ensure contacts directory and trusted sources directory exist
    let contacts_dir = vault_path.join(".footnotes").join("contacts");
    let trusted_user_dir = vault_path.join("footnotes").join(petname);
    fs::create_dir_all(&contacts_dir)?;
    fs::create_dir_all(&trusted_user_dir)?;

    fs::write(&contact_path, serde_json::to_string_pretty(&new_contact)?)?;
    eprintln!("Contact saved to {}", contact_path.display());

    eprintln!("\nImport complete!");
    eprintln!("Trusted user '{}' contact updated", petname);

    Ok(())
}

/// Import a user's contact information
pub async fn import(
    vault_path: &std::path::Path,
    file_path: &str,
    petname: &str,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)?;
    import_from_string(vault_path, &content, petname).await
}

/// Read and display all users and their devices
pub async fn read(vault_path: &std::path::Path) -> anyhow::Result<()> {
    let mut users = Vec::new();

    // Read "me" user from contact.json
    let mut me_devices = Vec::new();
    let mut master_public_key = None;
    let mut nickname = None;
    let contact_path = vault_path.join(".footnotes").join("contact.json");

    if contact_path.exists() {
        let contact_content = fs::read_to_string(&contact_path)?;
        if let Ok(contact_record) = serde_json::from_str::<crypto::ContactRecord>(&contact_content)
        {
            master_public_key = Some(contact_record.master_public_key.clone());
            nickname = if contact_record.nickname.is_empty() {
                None
            } else {
                Some(contact_record.nickname.clone())
            };

            for device in &contact_record.devices {
                me_devices.push(Device {
                    name: device.device_name.clone(),
                    endpoint_id: device.iroh_endpoint_id.clone(),
                    authorized_by: contact_record.master_public_key.clone(),
                    timestamp: device.added_at.clone(),
                    signature_valid: true,
                });
            }
        }
    }

    users.push(User {
        name: "me".to_string(),
        master_public_key,
        nickname,
        devices: me_devices,
    });

    // Scan .footnotes/contacts/ directory for trusted user contact files
    let contacts_dir = vault_path.join(".footnotes").join("contacts");
    if contacts_dir.exists() {
        for entry in fs::read_dir(&contacts_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Look for *.json files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let petname = path.file_stem().unwrap().to_string_lossy().to_string();

                // Read and parse the contact record
                let content = fs::read_to_string(&path)?;
                if let Ok(contact_record) = serde_json::from_str::<crypto::ContactRecord>(&content)
                {
                    // Verify signature
                    let signature_valid =
                        crypto::verify_contact_signature(&contact_record).unwrap_or(false);

                    if signature_valid {
                        // Convert devices to Device struct
                        let mut devices = Vec::new();
                        for device in &contact_record.devices {
                            devices.push(Device {
                                name: device.device_name.clone(),
                                endpoint_id: device.iroh_endpoint_id.clone(),
                                authorized_by: contact_record.master_public_key.clone(),
                                timestamp: device.added_at.clone(),
                                signature_valid: true,
                            });
                        }

                        users.push(User {
                            name: petname,
                            master_public_key: Some(contact_record.master_public_key),
                            nickname: if contact_record.nickname.is_empty() {
                                None
                            } else {
                                Some(contact_record.nickname)
                            },
                            devices,
                        });
                    }
                }
            }
        }
    }

    // Serialize and print as YAML
    let output = UsersOutput { users };
    let yaml = serde_yaml::to_string(&output)?;
    println!("{}", yaml);

    Ok(())
}

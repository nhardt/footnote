use crate::ui::components::contact::Contact;
use crate::ui::context::VaultContext;
use dioxus::prelude::*;

#[component]
pub fn ContactsScreen() -> Element {
    let mut trusted_contacts =
        use_signal(|| Vec::<(String, crate::core::crypto::ContactRecord)>::new());
    let vault_ctx = use_context::<VaultContext>();

    // Load contacts on mount
    use_effect(move || {
        let vault_ctx = vault_ctx.clone();
        spawn(async move {
            let vault_path = match vault_ctx.get_vault() {
                Some(path) => path.join(".footnotes"),
                None => return,
            };

            // Load trusted contacts
            let contacts_dir = vault_path.join("contacts");
            if let Ok(entries) = std::fs::read_dir(contacts_dir) {
                let mut contacts = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.ends_with(".json") {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if let Ok(contact) = serde_json::from_str::<
                                    crate::core::crypto::ContactRecord,
                                >(&content)
                                {
                                    let petname = file_name.trim_end_matches(".json").to_string();
                                    contacts.push((petname, contact));
                                }
                            }
                        }
                    }
                }
                contacts.sort_by(|a, b| a.0.cmp(&b.0));
                trusted_contacts.set(contacts);
            }
        });
    });

    rsx! {
        div { class: "max-w-4xl mx-auto p-6",
            // Contacts section
            div {
                h2 { class: "text-xl font-bold text-zinc-100 mb-4", "Contacts" }
                if trusted_contacts().is_empty() {
                    div { class: "text-zinc-400 italic", "No contacts yet" }
                } else {
                    div { class: "space-y-2",
                        for (petname, contact) in trusted_contacts().iter() {
                            Contact {
                                key: "{petname}",
                                petname: petname.clone(),
                                username: contact.username.clone(),
                                device_count: contact.devices.len(),
                            }
                        }
                    }
                }
            }
        }
    }
}

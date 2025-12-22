use dioxus::prelude::*;

use dioxus_clipboard::prelude::use_clipboard;
#[component]
pub fn Profile() -> Element {
    rsx! {
       "Profile",
        button {
            onclick: {
                let vault_ctx = vault_ctx.clone();
                move |_| {
                    let vault_ctx = vault_ctx.clone();
                    spawn(async move {
                        let vault_path = match vault_ctx.get_vault() {
                            Some(path) => path,
                            None => {
                                tracing::error!("No vault path available");
                                return;
                            }
                        };

                        if let Err(e) = copy_contact_record_clipboard(&vault_path).await {
                            tracing::error!("Failed to copy contact record: {}", e);
                        }
                    });
                }
            },
            "Share Contact"
        }

    }
}

async fn copy_contact_record_clipboard(vault_path: &std::path::Path) -> anyhow::Result<()> {
    let json_str = export_me_json_pretty(vault_path).await?;
    let mut clipboard = use_clipboard();
    let _ = clipboard.set(json_str);
    Ok(())
}

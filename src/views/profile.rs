use dioxus::prelude::*;

use dioxus_clipboard::prelude::use_clipboard;
#[component]
pub fn Profile() -> Element {
    rsx! {
       "Profile",
    }
}

async fn copy_contact_record_clipboard() -> anyhow::Result<()> {
    let json_str = ""; // export_me_json_pretty(vault_path).await?;
    let mut clipboard = use_clipboard();
    let _ = clipboard.set(json_str.to_string());
    Ok(())
}

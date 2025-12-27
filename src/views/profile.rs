use dioxus::prelude::*;
use dioxus_clipboard::prelude::use_clipboard;

#[component]
pub fn Profile() -> Element {
    rsx! {
        div { class: "max-w-2xl",
            h1 { class: "text-3xl font-bold", "Profile" }

            h2 { class: "text-2xl font-bold", "Contact" }

            div { class: "grid grid-cols-2",
                label { "username:" }
                input { class: "border-2", r#type: "text" }

                label { "primary device url:" }
                input { class: "border-2", r#type: "text" }
            }

            h2 { class: "text-2xl font-bold", "Devices" }

            div { class: "grid grid-cols-2",
                span { "desktop" }
                span { "endpoint-abc123" }

                span { "laptop" }
                span { "endpoint-abc123" }
            }
        }
    }
}

async fn copy_contact_record_clipboard() -> anyhow::Result<()> {
    let json_str = ""; // export_me_json_pretty(vault_path).await?;
    let mut clipboard = use_clipboard();
    let _ = clipboard.set(json_str.to_string());
    Ok(())
}

use dioxus::prelude::*;

use dioxus_clipboard::prelude::use_clipboard;
#[component]
pub fn Profile() -> Element {
    rsx! {
        div { class: "flex flex-1 flex-col bg-indigo-200 p-4",
            div { class: "space-y-4",
                div { class: "flex items-center gap-2",
                    label { class: "w-20", "Username:" }
                    input { class: "border-2 px-2 py-1", r#type: "text" }
                }
                div { class: "flex flex-col gap-2",
                    div { class: "font-semibold", "Devices:" }
                    div { class: "space-y-1",
                        div { class: "flex justify-between border-b py-1",
                            span { "iPhone 15" }
                            span { class: "text-gray-600", "endpoint-abc123" }
                        }
                        div { class: "flex justify-between border-b py-1",
                            span { "MacBook Pro" }
                            span { class: "text-gray-600", "endpoint-def456" }
                        }
                    }
                }
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

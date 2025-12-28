use crate::{context::VaultContext, model::vault::Vault};
use dioxus::prelude::*;
use dioxus_clipboard::prelude::use_clipboard;
use footnote::model::user::LocalUser;

#[component]
pub fn Profile() -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");

    // probably will need to make like:
    // if vault.is_primary() {
    //     LocalUserSection = LocalUserView()
    // }
    let local_user = LocalUser::new(&vault_path).expect("No local user record");
    let (public_key, username) = local_user
        .id_key_read()
        .expect("local user not initialized");
    let (device_public_key, device_name) = local_user
        .device_key_pub_read()
        .expect("local user not initialized");

    let mut username_value = use_signal(|| username.clone());
    let mut device_name_value = use_signal(|| device_name.clone());
    let mut err_message = use_signal(|| String::new());

    let devices = vault
        .device_read()
        .expect("failed to read any local devices");

    let save_username = move |_| {
        let username_update = username_value.read();
        if let Err(e) = local_user.username_update(&*username_update) {
            err_message.set(format!("err updating username: {}", e));
        }
    };

    // ok, here's a plan for the local device record, vault, local-user:
    // - by default, make vault
    // - device name update:
    //   - user.json does not exist: can edit local device name
    //   - user.json exists, we are primary: can edit device names
    //   - user.json exists, we are not primary: our local device name should be
    //     pulled from user.json
    // - device read:
    //   - user.json exists: return all. our local device should be there
    //   - no user.json: just return our local device. it's name is editable
    //   - on vault, since it applies whether or not we are primary
    //
    // vault states:
    // - None
    // - Secondary: can always init this way. can join or become primary
    //   - Join -> Gains user.json from primary, no id_key
    //   - ToPrimary -> Gains user.json from primary, id_key, id_key matches signing key
    // - Primary
    //   - Reset -> Secondary
    //
    // my outstanding concern with this is that both sides will have a home.md, with LWW
    // one will be blown away. possibly home.md could be special. maybe this use case:
    // A -> None -> Primary -> Write("home.md") -> Secondary
    // B -> None -> Primary
    // A -> Join(B)
    // isn't that important. the way it might be important is if users are confused about
    // Vault states, or even just generally the p2p sync'ing semantics.

    rsx! {
        div { class: "flex flex-col h-full w-2xl gap-2",
            h2 { class: "text-2xl font-bold", "Local Device Info" }

            div { class: "grid grid-cols-[auto_1fr_auto] gap-x-2 gap-y-4",
                label { "Username" }
                input {
                    class: "border-1 px-2",
                    r#type: "text",
                    value: "{username_value}",
                    oninput: move |e| username_value.set(e.value()),
                }
                button { class: "border-1 rounded px-2", onclick: save_username, "Update username" }

                label { "This Device Name" }
                input {
                    class: "border-1 px-2",
                    r#type: "text",
                    value: "{device_name_value}",
                    oninput: move |e| device_name_value.set(e.value()),
                }
                button { class: "border-1 rounded px-2", "Update local device name" }
            }

            h2 { class: "text-2xl font-bold", "Devices" }

            div { class: "grid grid-cols-2",
                for device in devices.iter() {
                    span { "{device.name}" }
                    span { class: "truncate", "{device.iroh_endpoint_id}" }
                }
            }
            label { "{err_message}" }
            button { class: "border-1 rounded", "Listen For Secondary Device" }
            button { class: "border-1 rounded", "Copy Device Join URL To Clipboard" }
            button { class: "border-1 rounded", "Copy Contact Record to Cliboard" }
        }
    }
}

async fn copy_to_clipboard(txt: &str) -> anyhow::Result<()> {
    let mut clipboard = use_clipboard();
    let _ = clipboard.set(txt.to_string());
    Ok(())
}

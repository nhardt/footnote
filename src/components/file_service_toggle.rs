use crate::context::VaultContext;
use crate::service::sync_service::SyncService;
use dioxus::prelude::*;
use footnote::model::vault::Vault;
use tokio_util::sync::CancellationToken;

#[component]
pub fn FileServiceToggle() -> Element {
    let mut cancel_token = use_signal(|| CancellationToken::new());
    let mut listening = use_signal(|| false);
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");

    let toggle_listener = move |_| {
        if listening() {
            cancel_token.read().cancel();
            listening.set(false);
        } else {
            let new_token = CancellationToken::new();
            cancel_token.set(new_token.clone());
            let vault = vault_ctx.get_vault().unwrap();
            spawn(async move {
                let _ = SyncService::listen(&vault, new_token).await;
            });
            listening.set(true);
        }
    };
    rsx! {
        button {
            class: "border-1 rounded",
            onclick: toggle_listener,
            if listening() { "Listening!" } else { "Not Listening" }
        }
    }
}

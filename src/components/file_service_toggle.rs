use std::path::Path;

use crate::context::VaultContext;
use crate::service::sync_service::SyncService;
use dioxus::prelude::*;
use footnote::{
    model::vault::Vault,
    service::{replica_service::ReplicaService, share_service::ShareService},
};
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

#[component]
pub fn FileServiceToggle() -> Element {
    let mut listening = use_signal(|| false);
    let mut send_cancel_token = use_signal(|| CancellationToken::new());
    let mut listen_cancel_token = use_signal(|| CancellationToken::new());

    let toggle_listener = move |_| {
        if listening() {
            listen_cancel_token.read().cancel();
            send_cancel_token.read().cancel();
            listening.set(false);
        } else {
            let vault_ctx = use_context::<VaultContext>();
            let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
            send_cancel_token.set(CancellationToken::new());
            listen_cancel_token.set(CancellationToken::new());

            let listen_vault_clone = vault_path.clone();
            spawn(async move {
                let _ =
                    SyncService::listen(&listen_vault_clone, listen_cancel_token.read().clone())
                        .await;
            });
            listening.set(true);

            let send_vault_clone = vault_path.clone();
            spawn(async move {
                push_changes(&send_vault_clone, send_cancel_token.read().clone()).await;
            });
        }
    };
    rsx! {
        button {
            class: "border-1 rounded",
            onclick: toggle_listener,
            if listening() { "Sync Active!" } else { "Sync Inactive" }
        }
    }
}

async fn push_changes(vault_path: &Path, cancel_token: CancellationToken) {
    let mut sync_interval = interval(Duration::from_secs(60));
    let vault = Vault::new(vault_path).expect("missing vault");
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                break;
            }
            _ = sync_interval.tick() => {
                let devices = vault.device_read().unwrap_or_default();
                for device in devices {
                    if let Err(e) = ReplicaService::push(&vault, &device.name).await {
                        eprintln!("Failed to sync to {}: {}", device.name, e);
                    }
                }
                let contacts = vault.contact_read().unwrap_or_default();
                for contact in contacts {
                    if let Err(e) = ShareService::share_with(&vault, &contact.nickname).await {
                        eprintln!("Failed to share with {}: {}", contact.nickname, e);
                    }
                }
            }
        }
    }
}

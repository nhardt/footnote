use std::path::Path;

use crate::context::VaultContext;
use crate::service::sync_service::SyncService;
use crate::{
    model::vault::Vault,
    service::{replica_service::ReplicaService, share_service::ShareService},
};
use dioxus::prelude::*;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

#[component]
pub fn FileServiceToggle() -> Element {
    let mut listening = use_signal(|| false);
    let mut send_cancel_token_signal = use_signal(|| CancellationToken::new());
    let mut listen_cancel_token_signal = use_signal(|| CancellationToken::new());

    let toggle_listener = move |_| {
        if listening() {
            tracing::info!("stopping send/listen activity");
            listen_cancel_token_signal.read().cancel();
            send_cancel_token_signal.read().cancel();
            listening.set(false);
        } else {
            let vault = use_context::<VaultContext>().get();

            let listen_cancel_token = CancellationToken::new();
            let listen_token_clone = listen_cancel_token.clone();
            let listen_vault_clone = vault.clone();
            tokio::spawn(async move {
                tracing::info!("spawning listen thread");
                let _ = SyncService::listen(listen_vault_clone, listen_token_clone).await;
            });
            listen_cancel_token_signal.set(listen_cancel_token);
            listening.set(true);

            let send_cancel_token = CancellationToken::new();
            let send_token_clone = send_cancel_token.clone();
            let send_vault_clone = vault.clone();
            tokio::spawn(async move {
                tracing::info!("spawning change push thread");
                push_changes(send_vault_clone, send_token_clone).await;
            });
            send_cancel_token_signal.set(send_cancel_token);
        }
    };
    rsx! {
        button {
            class: "text-sm font-mono text-zinc-400",
            onclick: toggle_listener,
            if listening() { "Syncing" } else { "Isolated" }
        }
    }
}

async fn push_changes(vault: Vault, cancel_token: CancellationToken) {
    tracing::info!("start push changes loop");
    let mut sync_interval = interval(Duration::from_secs(60));
    sync_interval.tick().await;
    loop {
        tracing::info!("tokio::select");
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracing::info!("tokio::select:: push_changes cancelled");
                break;
            }
            _ = sync_interval.tick() => {
                tracing::info!("tokio::select:: tick");
                let devices = vault.device_read().unwrap_or_default();
                for device in devices {
                    tracing::info!("attempting to push changes to {}", device.name);
                    if let Err(e) = ReplicaService::push(&vault, &device.name).await {
                        tracing::info!("Failed to sync to {}: {}", device.name, e);
                    }
                }
                let contacts = vault.contact_read().unwrap_or_default();
                for contact in contacts {
                    tracing::info!("attempting to share with {}", contact.nickname);
                    if let Err(e) = ShareService::share_with(&vault, &contact.nickname).await {
                        eprintln!("Failed to share with {}: {}", contact.nickname, e);
                    }
                }
            }
        }
    }
}

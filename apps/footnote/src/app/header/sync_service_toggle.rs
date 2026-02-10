use dioxus::prelude::*;

use iroh::Endpoint;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

use footnote_core::model::vault::Vault;
use footnote_core::service::sync_service::SyncService;
use footnote_core::service::ALPN_SYNC;

use crate::context::AppContext;

#[component]
pub fn SyncServiceToggle() -> Element {
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
            listening.set(true);

            let listen_cancel_token = CancellationToken::new();
            let listen_token_clone = listen_cancel_token.clone();
            listen_cancel_token_signal.set(listen_cancel_token);

            let send_cancel_token = CancellationToken::new();
            let send_token_clone = send_cancel_token.clone();
            send_cancel_token_signal.set(send_cancel_token);

            let app_context = use_context::<AppContext>();
            let vault = app_context.vault.read();
            let Ok((secret_key, _)) = vault.device_secret_key() else {
                tracing::warn!("could not get secret key");
                return;
            };
            let send_vault_clone = vault.clone();
            let listen_vault_clone = vault.clone();

            tokio::spawn(async move {
                let Ok(endpoint) = Endpoint::builder()
                    .secret_key(secret_key.clone())
                    .alpns(vec![ALPN_SYNC.to_vec()])
                    .bind()
                    .await
                else {
                    tracing::warn!("could not create endpoint");
                    return;
                };

                let endpoint_clone = endpoint.clone();
                tokio::spawn(async move {
                    tracing::info!("spawning listen thread");
                    let _ =
                        SyncService::listen(listen_vault_clone, endpoint_clone, listen_token_clone)
                            .await;
                });

                let endpoint_clone = endpoint.clone();
                tokio::spawn(async move {
                    tracing::info!("spawning change push thread");
                    push_changes(send_vault_clone, endpoint_clone, send_token_clone).await;
                });
            });
        }
    };
    rsx! {
        button {
            class: "flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
            onclick: toggle_listener,
            div {
                class: if listening() { "h-2.5 w-2.5 rounded-full bg-emerald-500" } else { "h-2.5 w-2.5 rounded-full bg-zinc-600" }
            }
            "Sync"
        }
    }
}

async fn push_changes(vault: Vault, endpoint: Endpoint, cancel_token: CancellationToken) {
    tracing::info!("start push changes loop");
    let mut sync_interval = interval(Duration::from_secs(60));
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
                    if let Err(e) = SyncService::mirror_to_device(&vault, endpoint.clone(), &device.name).await {
                        tracing::warn!("Failed to sync to {}: {}", device.name, e);
                    }
                }
                let contacts = vault.contact_read().unwrap_or_default();
                for contact in contacts {
                    tracing::info!("attempting to share with {}", contact.nickname);
                    if let Err(e) = SyncService::share_to_contact(&vault, endpoint.clone(), &contact.nickname).await {
                        tracing::warn!("Failed to share with {}: {}", contact.nickname, e);
                    }
                }
            }
        }
    }
}

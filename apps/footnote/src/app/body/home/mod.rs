use dioxus::prelude::*;

use footnote_core::model::vault::VaultState;

use crate::context::{AppContext, MenuContext};

mod sync_activity;
use sync_activity::SyncActivity;

#[component]
pub fn Home() -> Element {
    let mut app_context = use_context::<AppContext>();
    let vault_state = app_context.vault_state;

    rsx! {
        div {
            class: "flex-1 overflow-y-auto",
            div {
                class: "max-w-3xl mx-auto px-6 py-12",

                div {
                    class: "mb-12",
                    h1 {
                        class: "text-2xl font-bold font-mono text-zinc-100 mb-2",
                        "Footnote"
                    }
                    p {
                        class: "text-sm text-zinc-400",
                        "Use the upper left menu to start taking notes"
                    }
                }

                div {
                    class: "space-y-8",

                    match *vault_state.read() {
                        VaultState::StandAlone | VaultState::Uninitialized => rsx! {
                            div {
                                class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-6",
                                h2 {
                                    class: "text-sm font-semibold font-mono text-zinc-400 mb-3",
                                    "Device Setup"
                                }
                                p {
                                    class: "text-xs text-zinc-500 mb-4",
                                    "The First Device will manage your list of
                                    devices. Mobile is recommended. A mobile
                                    device can pair by scanning a QR code, and
                                    can exchange contact records via AirDrop or
                                    Nearby Share"
                                }
                                div {
                                    class: "flex gap-2",
                                    button {
                                        class: "flex-1 px-4 py-3 text-center text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 rounded-lg transition-colors border border-zinc-800",
                                        onclick:  move |_| {
                                            if app_context.vault.read().transition_to_primary("default", "primary").is_ok() {
                                                if let Err(e) = app_context.reload() {
                                                    tracing::warn!("failed to reload app: {}", e);
                                                }
                                            }
                                        },
                                        "This is my First Device"
                                    }
                                    button {
                                        class: "flex-1 px-4 py-3 text-center text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 rounded-lg transition-colors border border-zinc-800",
                                        onclick: move |_| consume_context::<MenuContext>().set_listen_for_pair_visible(),
                                        "I already have a First Device"
                                    }
                                }
                            }
                        },
                        _ => rsx! { SyncActivity {} }
                    }
                }
            }
        }
    }
}

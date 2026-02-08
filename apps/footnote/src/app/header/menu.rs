use dioxus::prelude::*;

use footnote_core::model::vault::{Vault, VaultState};

use crate::context::AppContext;
use crate::route::Route;

use crate::modal::listen_for_pair_modal::ListenForPairModalVisible;
use crate::modal::pair_with_listening_device_modal::PairWithListeningDeviceModalVisible;
use crate::modal::share_my_contact_modal::ShareMyContactModalVisible;

#[component]
pub fn HeaderMenu() -> Element {
    let nav = use_navigator();
    let mut app_context = use_context::<AppContext>();
    let vault_state = app_context.vault_state;
    let mut menu_visible = use_signal(|| false);
    let mut show_new_note_modal = use_signal(|| false);

    rsx! {
        button {
            class: "p-2 -ml-2 hover:bg-zinc-800 rounded-lg transition-colors",
            onclick: move |_| menu_visible.set(true),
            aria_label: "Menu",
            svg {
                class: "w-5 h-5",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    d: "M4 6h16M4 12h16M4 18h16",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                }
            }
        }

        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm z-40",
            onclick: move |_| menu_visible().set(false),
        }

        div {
            class: "fixed inset-y-0 left-0 w-72 bg-zinc-900 border-r border-zinc-800 z-50 shadow-2xl flex flex-col",
            onclick: move |e| e.stop_propagation(),

            div {
                class: "p-4 border-b border-zinc-800",
                h2 {
                    class: "text-lg font-semibold text-zinc-100",
                    "Footnote"
                }
            }

            div {
                class: "p-2 border-t border-zinc-800",

                MenuButton {
                    label: "Home",
                    onclick: move |_| {
                        nav.push(Route::Home {});
                        menu_visible().set(false);
                    }
                }

                match *vault_state.read() {

                    VaultState::Primary => rsx! {
                        MenuButton {
                            label: "Device List",
                            onclick: move |_| {
                                nav.push(Route::Profile {});
                                menu_visible().set(false);
                            }
                        }

                        MenuButton {
                            label: "Add Listening Device",
                            onclick: move |_| {
                                consume_context::<PairWithListeningDeviceModalVisible>().set(true);
                                menu_visible().set(false);
                            }
                        }

                        MenuButton {
                            label: "Share Contact Record",
                            onclick: move |_| {
                                consume_context::<ShareMyContactModalVisible>().set(true);
                                menu_visible().set(false);
                            }
                        }

                        MenuButton {
                            label: "Import Contact",
                            onclick: move |_| {
                                consume_context::<ImportContactModalVisible>().set(true);
                                menu_visible.set(false);
                            },
                        }

                        MenuButton {
                            label: "Contacts",
                            onclick: move |_| {
                                nav.push(Route::ContactBrowser {});
                                menu_visible().set(false);
                            }
                        }

                        MenuButton {
                            label: "Debug: Reset to Standalone",
                            onclick: move |_| {
                                let mut app_context = use_context::<AppContext>();
                                if app_context.vault.read().transition_to_standalone().is_ok() {
                                    if let Err(e) = app_context.reload() {
                                        tracing::warn!("failed to reload app: {}", e);
                                    }
                                }
                                menu_visible().set(false);
                            }
                        }
                    },

                    VaultState::SecondaryJoined => rsx! {
                        MenuButton {
                            label: "Share Contact Record*",
                            onclick: move |_| {
                                consume_context::<ShareMyContactModalVisible>().set(true);
                                menu_visible().set(false);
                            }
                        }

                        MenuButton {
                            label: "Device List*",
                            onclick: move |_| {
                                nav.push(Route::Profile {});
                                menu_visible().set(false);
                            }
                        }

                        MenuButton {
                            label: "Contacts*",
                            onclick: move |_| {
                                nav.push(Route::ContactBrowser {});
                                menu_visible().set(false);
                            }
                        }

                        MenuButton {
                            label: "Debug: Reset to Standalone",
                            onclick: move |_| {
                                let mut app_context = use_context::<AppContext>();
                                if app_context.vault.read().transition_to_standalone().is_ok() {
                                    if let Err(e) = app_context.reload() {
                                        tracing::warn!("failed to reload app, probably should crash: {}", e);
                                    }
                                }
                                menu_visible().set(false);
                            }
                        }


                    },

                    _ => rsx! {
                        MenuButton {
                            label: "Create Device Group",
                            onclick: move |_| {
                                if app_context
                                    .vault
                                    .read()
                                    .transition_to_primary("default", "primary")
                                    .is_ok()
                                {
                                    if let Err(e) = app_context.reload() {
                                        tracing::warn!("failed to reload app: {}", e);
                                    }
                                }
                                menu_visible().set(false);
                            }
                        }
                        MenuButton {
                            label: "Join Device Group",
                            onclick: move |_| {
                                consume_context::<ListenForPairModalVisible>().set(true);
                                menu_visible().set(false);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MenuButton(label: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "w-full px-4 py-3 text-left text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 rounded-lg transition-colors",
            onclick: onclick,
            "{label}"
        }
    }
}

#[component]
pub fn MenuDivider() -> Element {
    rsx! {
        div {
            class: "my-2 border-t border-zinc-800"
        }
    }
}

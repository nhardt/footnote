use crate::{
    components::{
        listen_for_pair_modal::ListenForPairModalVisible,
        pair_with_listening_device_modal::{
            ListeningDeviceUrl, PairWithListeningDeviceModalVisible,
        },
        share_my_contact_modal::ShareMyContactModalVisible,
    },
    context::AppContext,
    Route,
};
use footnote_core::model::vault::VaultState;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct AppMenuVisible(pub Signal<bool>);

impl AppMenuVisible {
    pub fn set(&mut self, value: bool) {
        self.0.set(value);
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

#[component]
pub fn AppMenu(visible: bool, on_close: EventHandler, children: Element) -> Element {
    let nav = use_navigator();
    let mut app_context = use_context::<AppContext>();
    let vault_state = app_context.vault_state;

    if !visible {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm z-40",
            onclick: move |_| on_close.call(()),
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

            nav {
                class: "flex-1 overflow-y-auto p-2",

                MenuButton {
                    label: "Home",
                    onclick: move |_| {
                        nav.push(Route::NoteDefault {});
                        on_close.call(());
                    }
                }

                {children}
            }

            div {
                class: "p-2 border-t border-zinc-800",

                MenuButton {
                    label: "Notes",
                    onclick: move |_| {
                        nav.push(Route::NoteDefault {});
                        on_close.call(());
                    }
                }

                match *vault_state.read() {

                    VaultState::Primary => rsx! {
                        MenuButton {
                            label: "Device List",
                            onclick: move |_| {
                                nav.push(Route::Profile {});
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }

                        MenuButton {
                            label: "Add Listening Device",
                            onclick: move |_| {
                                consume_context::<ListeningDeviceUrl>().set("".to_string());
                                consume_context::<PairWithListeningDeviceModalVisible>().set(true);
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }

                        MenuButton {
                            label: "Share Contact Record",
                            onclick: move |_| {
                                consume_context::<ShareMyContactModalVisible>().set(true);
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }

                        MenuButton {
                            label: "Contacts",
                            onclick: move |_| {
                                nav.push(Route::ContactBrowser {});
                                consume_context::<AppMenuVisible>().set(false);
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
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }
                    },

                    VaultState::SecondaryJoined => rsx! {
                        MenuButton {
                            label: "Share Contact Record*",
                            onclick: move |_| {
                                consume_context::<ShareMyContactModalVisible>().set(true);
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }

                        MenuButton {
                            label: "Device List*",
                            onclick: move |_| {
                                nav.push(Route::Profile {});
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }

                        MenuButton {
                            label: "Contacts*",
                            onclick: move |_| {
                                nav.push(Route::ContactBrowser {});
                                consume_context::<AppMenuVisible>().set(false);
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
                                consume_context::<AppMenuVisible>().set(false);
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
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }
                        MenuButton {
                            label: "Join Device Group",
                            onclick: move |_| {
                                consume_context::<ListenForPairModalVisible>().set(true);
                                consume_context::<AppMenuVisible>().set(false);
                            }
                        }
                    }
                }
            }
        }
    }
}

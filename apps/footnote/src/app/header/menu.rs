use dioxus::prelude::*;

use footnote_core::model::vault::VaultState;

use crate::context::AppContext;
use crate::context::MenuContext;

#[component]
pub fn HeaderMenu() -> Element {
    let mut app_context = use_context::<AppContext>();
    let mut menu_context = use_context::<MenuContext>();
    let vault_state = app_context.vault_state;

    rsx! {
        div {
            class: "relative",

            button {
                class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                onclick: move |_| menu_context.toggle_menu(),
                "☰"
            }

            button { class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                onclick: move |_| consume_context::<MenuContext>().go_back(),
                "←"
            }

            button { class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                onclick: move |_| consume_context::<MenuContext>().go_forward(),
                "→"
            }

            if *menu_context.menu_visible.read() {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| menu_context.close_all(),
                }

                div {
                    class: "absolute top-full left-0 mt-1 w-64 bg-zinc-900 border border-zinc-700 rounded-md shadow-2xl z-50",
                    onclick: move |e| e.stop_propagation(),

                    div {
                        class: "p-3 border-b border-zinc-800",
                        h2 {
                            class: "text-lg font-bold font-mono",
                            "Footnote"
                        }
                    }

                    div {
                        class: "py-2",

                        MenuButton {
                            label: "Home",
                            onclick: move |_| { consume_context::<MenuContext>().go_home(); }
                        }

                        MenuButton {
                            label: "New Note",
                            onclick: move |_| { consume_context::<MenuContext>().set_new_note_visible(); }
                        }

                        MenuButton {
                            label: "Browse",
                            onclick: move |_| { consume_context::<MenuContext>().set_note_browser_visible(); }
                        }

                        match *vault_state.read() {
                            VaultState::Primary => rsx! {
                                MenuButton {
                                    label: "Add Listening Device",
                                    onclick: move |_| { consume_context::<MenuContext>().set_pair_with_listener_visible(&"".to_string()); }
                                }

                                MenuButton {
                                    label: "Share Contact Record",
                                    onclick: move |_| { consume_context::<MenuContext>().set_share_contact_visible(); }
                                }

                                MenuButton {
                                    label: "Profile",
                                    onclick: move |_| { consume_context::<MenuContext>().go_profile(); }
                                }

                                MenuButton {
                                    label: "Import Contact",
                                    onclick: move |_| { consume_context::<MenuContext>().set_import_contact_visible(&"".to_string()); }
                                }

                                MenuButton {
                                    label: "Contacts",
                                    onclick: move |_| { consume_context::<MenuContext>().go_contacts(); }
                                }

                                MenuDivider {}

                                MenuButton {
                                    label: "Debug: Reset to Standalone",
                                    onclick: move |_| {
                                        let mut app_context = use_context::<AppContext>();
                                        if app_context.vault.read().transition_to_standalone().is_ok() {
                                            if let Err(e) = app_context.reload() {
                                                tracing::warn!("failed to reload app: {}", e);
                                            }
                                        }
                                        consume_context::<MenuContext>().close_all();
                                    }
                                }
                            },

                            VaultState::SecondaryJoined => rsx! {
                                MenuButton {
                                    label: "Share Contact Record*",
                                    onclick: move |_| { consume_context::<MenuContext>().set_share_contact_visible(); }
                                }

                                MenuButton {
                                    label: "Profile*",
                                    onclick: move |_| { consume_context::<MenuContext>().go_profile(); }
                                }

                                MenuButton {
                                    label: "Contacts*",
                                    onclick: move |_| { consume_context::<MenuContext>().go_contacts(); }
                                }

                                MenuDivider {}

                                MenuButton {
                                    label: "Debug: Reset to Standalone",
                                    onclick: move |_| {
                                        let mut app_context = use_context::<AppContext>();
                                        if app_context.vault.read().transition_to_standalone().is_ok() {
                                            if let Err(e) = app_context.reload() {
                                                tracing::warn!("failed to reload app: {}", e);
                                            }
                                        }
                                        consume_context::<MenuContext>().close_all();
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
                                        consume_context::<MenuContext>().close_all();
                                    }
                                }
                                MenuButton {
                                    label: "Join Device Group",
                                    onclick: move |_| {
                                        consume_context::<MenuContext>().set_listen_for_pair_visible();
                                    }
                                }
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
            class: "w-full px-4 py-2 text-left text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 transition-colors",
            onclick: onclick,
            "{label}"
        }
    }
}

#[component]
pub fn MenuDivider() -> Element {
    rsx! {
        div {
            class: "my-1 mx-2 border-t border-zinc-800"
        }
    }
}

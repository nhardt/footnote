use dioxus::prelude::*;

use crate::context::AppContext;

use footnote_core::model::contact::Contact;

#[component]
pub fn ShareDropdown(share_with: Signal<String>, on_change: EventHandler<()>) -> Element {
    let app_context = use_context::<AppContext>();
    let mut show_dropdown = use_signal(|| false);
    let mut sorted_contacts = use_signal(Vec::<Contact>::new);

    let current_shares: Vec<String> = share_with
        .read()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    rsx! {
        div {
            class: "relative",

            button {
                class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                onclick: move |_| {
                    if !show_dropdown() {
                        let contacts = app_context.contacts.read();
                        let shares: Vec<String> = share_with
                            .read()
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect();

                        let mut sorted: Vec<_> = contacts.iter().cloned().collect();
                        sorted.sort_by(|a, b| {
                            let a_selected = shares.contains(&a.nickname);
                            let b_selected = shares.contains(&b.nickname);
                            match (a_selected, b_selected) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.nickname.cmp(&b.nickname),
                            }
                        });
                        sorted_contacts.set(sorted);
                    }
                    show_dropdown.toggle();
                },
                "Share"
            }

            if show_dropdown() {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| show_dropdown.set(false),
                }

                div {
                    class: "absolute top-full left-0 mt-1 w-48 bg-zinc-900 border border-zinc-700 rounded-md shadow-2xl z-50",
                    onclick: move |e| e.stop_propagation(),

                    div {
                        class: "py-1",

                        for contact in sorted_contacts.read().iter() {
                            {
                                let nickname = contact.nickname.clone();
                                let is_selected = current_shares.contains(&nickname);
                                let name_for_click = nickname.clone();

                                rsx! {
                                    button {
                                        key: "{nickname}",
                                        class: "w-full px-3 py-2 text-left text-sm text-zinc-300 hover:bg-zinc-800 flex items-center gap-3",
                                        onclick: move |_| {
                                            let mut shares: Vec<String> = share_with
                                                .read()
                                                .split_whitespace()
                                                .map(|s| s.to_string())
                                                .collect();

                                            if shares.contains(&name_for_click) {
                                                shares.retain(|s| s != &name_for_click);
                                            } else {
                                                shares.push(name_for_click.clone());
                                            }

                                            share_with.set(shares.join(" "));
                                            on_change.call(());
                                        },
                                        div {
                                            class: "w-4 h-4 border rounded flex items-center justify-center",
                                            class: if is_selected { "bg-zinc-100 border-zinc-100" } else { "border-zinc-600" },
                                            if is_selected {
                                                span { class: "text-zinc-900 text-xs", "✓" }
                                            }
                                            else {
                                                ""
                                            }
                                        },
                                        span { "{nickname}" }
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

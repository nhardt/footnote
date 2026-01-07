use crate::{
    components::footnotes::Footnotes,
    context::AppContext,
    elements::primary_button::PrimaryButton,
    model::note::Note,
    util::{
        manifest::Manifest,
        tree_node::{build_tree_from_manifest, TreeNode},
    },
    Route,
};
use dioxus::{html::i, prelude::*};
use serde_yaml::from_str;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[component]
pub fn NoteView(file_path: String) -> Element {
    let app_context = use_context::<AppContext>();
    let loaded_from = urlencoding::decode(&file_path).unwrap().to_string();
    tracing::info!("loading {}", loaded_from);

    let loaded_from_clone = loaded_from.clone();
    let mut relative_path = use_signal(move || {
        let loaded_from_path = PathBuf::from(loaded_from_clone);
        loaded_from_path
            .strip_prefix(app_context.vault.read().base_path())
            .unwrap_or(&loaded_from_path)
            .to_string_lossy()
            .to_string()
    });
    let note = use_signal(move || {
        let full_path = PathBuf::from(loaded_from);
        let note_from_path = match Note::from_path(full_path, true) {
            Ok(n) => n,
            Err(_) => {
                Note::from_string("Failed to load", true).expect("Expected to make blank note")
            }
        };
        note_from_path
    });
    let mut share_with = use_signal(move || note.read().frontmatter.share_with.join(" "));
    let body = use_signal(move || note.read().content.clone());
    let footnotes = use_signal(move || note.read().footnotes.clone());
    let mut err_label = use_signal(|| String::new());

    let save_note = move |_| async move {
        let new_relative_path = PathBuf::from(relative_path.read().to_string());
        let new_full_path = app_context
            .vault
            .read()
            .base_path()
            .join(&*new_relative_path);

        let share_with_str = share_with.read();
        let share_with = share_with_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut note = note.read().cloned();
        note.frontmatter.share_with = share_with;
        let mut note_body_eval =
            document::eval("dioxus.send(document.getElementById('note-body').value)");

        match note_body_eval.recv::<String>().await {
            Ok(note_body) => {
                tracing::info!("saving note to {}", new_full_path.to_string_lossy());
                if let Err(e) = note.update(&new_full_path, &note_body) {
                    err_label.set(format!("Failed to save note: {e}"));
                }
            }
            Err(e) => {
                err_label.set(format!("JavaScript Eval Error: {e:?}"));
            }
        }
    };

    let mut select_note_visible = use_signal(|| false);
    let select_note = move |_| {
        select_note_visible.set(true);
    };
    let select_note_modal_oncancel = move |_| {
        select_note_visible.set(false);
    };
    let select_note_modal_onselect = move |_| {
        select_note_visible.set(false);
    };

    rsx! {
        div { class: "h-full flex flex-col flex-1",
            div { class: "border-b border-zinc-800 bg-zinc-900/30 px-6 py-4",
                div { class: "max-w-5xl mx-auto",
                    div { class: "grid grid-cols-[auto_1fr_auto] gap-x-3 gap-y-3 items-center",
                        label { class: "text-sm font-medium text-zinc-400", "File" }
                        input {
                            class: "px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            r#type: "text",
                            value: "{relative_path}",
                            oninput: move |e| relative_path.set(e.value()),
                        }
                        button { class: "px-4 py-1.5 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                            onclick: select_note,
                            "Open"
                        }
                        if select_note_visible() {
                            NoteSelectModal {
                                oncancel: select_note_modal_oncancel,
                                onselect: select_note_modal_onselect
                            }
                        }


                        label { class: "text-sm font-medium text-zinc-400", "Shared with" }
                        input {
                            class: "flex-1 px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            r#type: "text",
                            value: "{share_with}",
                            oninput: move |e| share_with.set(e.value())
                        }
                        button { class: "px-4 py-1.5 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                            onclick: save_note,
                            "Save"
                        }
                    }
                }
            }
            div { class: "h-full flex-1 overflow-hidden",
                div { class: "h-full max-w-5xl mx-auto px-6 py-6",
                    textarea {
                        id: "note-body",
                        class: "w-full h-full px-4 py-3 bg-zinc-900/30 border border-zinc-800 rounded-lg text-sm font-mono text-zinc-100 resize-none focus:border-zinc-700 focus:ring-1 focus:ring-zinc-700",
                        placeholder: "Once upon a time...",
                        initial_value: "{body}",
                    }
                }
            }

            div { class: "max-w-5xl mx-auto px-6 py-6",
                Footnotes {
                    footnotes: footnotes
                }
            }
        }
    }
}

#[component]
fn NoteSelectModal(onselect: EventHandler, oncancel: EventHandler<MouseEvent>) -> Element {
    let app_context = use_context::<AppContext>();
    let tree = use_memo(move || build_tree_from_manifest(&app_context.manifest.read()));

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-start justify-start p-8 z-50",

            // Browser container
            div {
                class: "w-[80dvw] h-[90vh] border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl flex flex-col",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div { class: "sticky top-0 py-3 px-4 border-b bg-zinc-900 border-zinc-800",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-sm font-semibold", "Browse Files" }
                        button {
                            class: "p-1 rounded transition-colors text-zinc-500 hover:text-zinc-300",
                            onclick: oncancel,
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M6 18L18 6M6 6l12 12",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }
                        }
                    }
                }

                div { class: "flex-1 overflow-y-auto p-2",
                    for (name, child) in tree().children {
                        TreeNodeView {
                            name: name,
                            node: child,
                            onselect: onselect,
                        }
                    }
                }

                div { class: "p-4 border-t border-zinc-800 justify-right",
                    PrimaryButton { onclick: oncancel, "Cancel" }
                }
            }
        }
    }
}

#[component]
fn TreeNodeView(name: String, node: TreeNode, onselect: EventHandler) -> Element {
    let is_folder = !node.children.is_empty();

    if is_folder {
        let mut sorted_children: Vec<_> = node.children.values().cloned().collect();
        sorted_children.sort_by(|a, b| a.name.cmp(&b.name));

        rsx! {
            BrowserRowFolder {
                name: node.name.clone(),
                open: false,
                for child in sorted_children {
                    TreeNodeView {
                        name: child.name.clone(),
                        node: child,
                        onselect: onselect
                    }
                }
            }
        }
    } else {
        rsx! {
            BrowserRowFile {
                node: node,
                onselect: onselect
            }
        }
    }
}

#[component]
fn BrowserRowFolder(name: String, open: bool, children: Element) -> Element {
    let mut open_signal = use_signal(|| open);
    let toggle_open = move |_| open_signal.set(!open_signal());

    rsx! {
        if open_signal() {
            button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick:toggle_open,
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        d: "M19 9l-7 7-7-7",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                    }
                }
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path { d: "M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" }
                }
                span { class: "font-medium", "{name}" }
            }
            div { class: "ml-6",
                {children}
            }
        }
        else
        {
            button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick:toggle_open,
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        d: "M9 5l7 7-7 7",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                    }
                }
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path { d: "M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" }
                }
                span { class: "font-medium", "{name}" }
            }
        }
    }
}

#[component]
fn BrowserRowFile(node: TreeNode, onselect: EventHandler) -> Element {
    let nav = use_navigator();
    let app_context = use_context::<AppContext>();
    let path_clone = node.full_path.clone();

    let onclick = move |_| {
        if let Some(relative_path) = &path_clone {
            tracing::info!("nav to {}", relative_path.to_string_lossy());
            let full_path = app_context
                .vault
                .read()
                .base_path()
                .join(relative_path)
                .to_string_lossy()
                .to_string();
            tracing::info!("full path {}", full_path);
            let encoded = urlencoding::encode(&full_path);

            nav.push(Route::NoteView {
                file_path: encoded.into_owned(),
            });

            onselect(());
        }
    };
    rsx! {
        button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
            onclick: onclick,
            div { class: "flex-shrink-0 w-4 h-4" }
            svg {
                class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                fill: "currentColor",
                view_box: "0 0 20 20",
                path {
                    clip_rule: "evenodd",
                    d: "M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z",
                    fill_rule: "evenodd",
                }
            }
            span { class: "text-zinc-300", "{node.name}" }
        }
    }
}

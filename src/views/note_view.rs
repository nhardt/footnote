use crate::{
    components::{footnotes::Footnotes, new_note_modal::NewNoteModal},
    context::AppContext,
    elements::primary_button::PrimaryButton,
    model::note::Note,
    util::{
        manifest::{create_manifest_local, Manifest},
        tree_node::{build_tree_from_manifest, TreeNode},
    },
    Route,
};
use dioxus::{document::EvalError, html::i, prelude::*};
use regex::bytes::Regex;
use serde_yaml::from_str;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[component]
pub fn NoteView(file_path: String) -> Element {
    let nav = navigator();
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

    let mut note = use_signal(move || {
        let full_path = PathBuf::from(loaded_from);
        let note_from_path = match Note::from_path(full_path, true) {
            Ok(n) => n,
            Err(_) => {
                Note::from_string("Failed to load", true).expect("Expected to make blank note")
            }
        };
        note_from_path
    });

    let read_only = use_signal(move || relative_path.read().starts_with("footnotes"));

    let mut share_with = use_signal(move || note.read().frontmatter.share_with.join(" "));
    let body = use_signal(move || note.read().content.clone());
    let footnotes = use_signal(move || note.read().footnotes.clone());
    let mut err_label = use_signal(|| String::new());

    let sync_body_to_footnotes = move |_| async move {
        let mut footnotes_signal = footnotes.clone();
        let mut footnotes_vec = footnotes.read().clone();

        let mut note_body_eval =
            document::eval("dioxus.send(document.getElementById('note-body').value)");
        let Ok(note_body) = note_body_eval.recv::<String>().await else {
            return;
        };

        let re = Regex::new(r"\[(\d+)\]").unwrap();
        let mut link_names: Vec<String> = Vec::new();
        for cap in re.captures_iter(note_body.as_bytes()) {
            let full_match = cap.get(0).unwrap();
            let link_name = std::str::from_utf8(&cap[1]).unwrap();
            let end_pos = full_match.end();
            if end_pos < note_body.len() && note_body.as_bytes()[end_pos] == b'(' {
                continue;
            }
            link_names.push(link_name.to_string());
        }

        for (_, [link_name]) in re.captures_iter(note_body.as_bytes()).map(|c| c.extract()) {
            let link_name = std::str::from_utf8(link_name).unwrap();
            link_names.push(link_name.to_string());
        }

        footnotes_vec.retain(|name, _| link_names.contains(&name));
        for name in link_names {
            footnotes_vec
                .entry((&name).to_string())
                .or_insert(String::new());
        }
        tracing::info!("sync'd {} footnotes", footnotes_vec.len());

        footnotes_vec
            .iter()
            .for_each(|i| tracing::info!("{} -> {}", i.0, i.1));
        footnotes_signal.set(footnotes_vec);
    };

    let save_link_to_footnote = move |(link_name, link_value)| async move {
        let mut footnotes_signal = footnotes.clone();
        let mut footnotes_vec = footnotes.read().clone();

        tracing::info!("saving {} -> {} to footnotes", link_name, link_value);
        footnotes_vec.insert(link_name, link_value);
        footnotes_vec
            .iter()
            .for_each(|i| tracing::info!("{} -> {}", i.0, i.1));
        footnotes_signal.set(footnotes_vec);
    };

    let save_note = move |_| async move {
        let footnotes_vec = footnotes.read().clone();
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

        let mut note_copy = note.read().clone();
        note_copy.frontmatter.share_with = share_with;

        let mut note_body_eval =
            document::eval("dioxus.send(document.getElementById('note-body').value)");

        match note_body_eval.recv::<String>().await {
            Ok(note_body) => {
                tracing::info!("saving note to {}", new_full_path.to_string_lossy());

                note_copy.content = note_body;
                note_copy.footnotes = footnotes_vec;

                if let Err(e) = note_copy.to_file(&new_full_path) {
                    err_label.set(format!("Failed to save note: {e}"));
                    return;
                }

                note.set(note_copy);
            }
            Err(e) => {
                err_label.set(format!("JavaScript Eval Error: {e:?}"));
            }
        }
    };

    let navigate_to_uuid = move |uuid: String| async move {
        tracing::info!("navigate_to_uuid: {}", uuid);
        let app_context = use_context::<AppContext>();
        if let Ok(uuid) = Uuid::parse_str(&uuid) {
            if let Some(entry) = app_context.manifest.read().get(&uuid) {
                tracing::info!(
                    "found entry for uuid, requesting nav to: {}",
                    entry.path.to_string_lossy()
                );
                nav.push(Route::NoteView {
                    file_path: urlencoding::encode(
                        &app_context
                            .vault
                            .read()
                            .base_path()
                            .join(&entry.path)
                            .to_string_lossy()
                            .to_string(),
                    )
                    .to_string(),
                });
            }
        } else {
            tracing::info!("could not convert uuid str {} to uuid", uuid);
        }
    };

    let mut show_new_note_modal = use_signal(|| false);
    let mut select_note_visible = use_signal(|| false);

    let select_note = move |_| {
        let mut app_context = use_context::<AppContext>();
        if let Ok(_) = app_context.reload_manifest() {
            select_note_visible.set(true);
        } else {
            tracing::info!("could not read filsystem");
        }
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
                div { class: "border-b border-zinc-800 bg-zinc-900/30 px-6 py-4",
                    div { class: "max-w-5xl mx-auto",
                        div { class: "grid grid-cols-[auto_1fr] gap-x-3 gap-y-3 items-center mb-4",
                            label { class: "text-sm font-medium text-zinc-400", "File" }
                            input {
                                class: "px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                r#type: "text",
                                value: "{relative_path}",
                                oninput: move |e| relative_path.set(e.value()),
                            }

                            label { class: "text-sm font-medium text-zinc-400", "Shared with" }
                            input {
                                class: "px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                r#type: "text",
                                value: "{share_with}",
                                oninput: move |e| share_with.set(e.value())
                            }
                        }

                        div { class: "flex gap-3 w-full",
                            button {
                                class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                onclick: move |_| nav.go_back(),
                                "←"
                            }
                            button {
                                class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                onclick: move |_| nav.go_forward(),
                                "→"
                            }
                            button {
                                class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                onclick: select_note,
                                "Open"
                            }
                            if select_note_visible() {
                                NoteSelectModal {
                                    oncancel: select_note_modal_oncancel,
                                    onselect: select_note_modal_onselect
                                }
                            }
                            button {
                                class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                onclick: move |_| show_new_note_modal.set(true),
                                "New Note"
                            }
                            if show_new_note_modal() {
                                NewNoteModal {
                                    oncancel: move |_| show_new_note_modal.set(false)
                                }
                            }
                            if !read_only() {
                                button {
                                    class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                    onclick: save_note,
                                    "Save"
                                }
                            }
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
                        onblur: sync_body_to_footnotes,
                        initial_value: "{body}",
                        readonly: read_only
                    }
                }
            }

            div { class: "max-w-5xl mx-auto px-6 py-6",
                Footnotes {
                    footnotes: footnotes,
                    onlink: save_link_to_footnote,
                    onuuidclick: navigate_to_uuid,
                }
            }
        }
    }
}

#[component]
fn NoteSelectModal(onselect: EventHandler, oncancel: EventHandler<MouseEvent>) -> Element {
    let app_context = use_context::<AppContext>();
    let tree = use_memo(move || build_tree_from_manifest(&app_context.manifest.read()));

    let mut root_children: Vec<_> = tree().children.into_iter().collect();
    root_children.sort_by(|(_, a), (_, b)| {
        let a_is_folder = !a.children.is_empty();
        let b_is_folder = !b.children.is_empty();
        match (a_is_folder, b_is_folder) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

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
                    for (name, child) in root_children {
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
        sorted_children.sort_by(|a, b| {
            let a_is_folder = !a.children.is_empty();
            let b_is_folder = !b.children.is_empty();

            match (a_is_folder, b_is_folder) {
                (true, false) => std::cmp::Ordering::Less, // folders first
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name), // then alphabetical within type
            }
        });

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

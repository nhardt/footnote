use crate::{
    components::{
        app_header::AppHeader,
        app_menu::{AppMenu, MenuButton},
        footnotes::Footnotes,
        new_note_modal::NewNoteModal,
        sync_service_toggle::SyncServiceToggle,
    },
    context::AppContext,
};
use footnote_core::model::note::Note;
use footnote_core::util::tree_node::{build_tree_from_manifest, TreeNode};
use dioxus::prelude::*;
use indexmap::IndexMap;
use regex::bytes::Regex;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
enum SaveStatus {
    Saved,
    Unsaved,
    Syncing,
}

#[component]
pub fn NoteView(file_path_segments: ReadSignal<Vec<String>>) -> Element {
    let nav = navigator();
    let app_context = use_context::<AppContext>();

    let mut menu_visible = use_signal(|| false);
    let mut show_new_note_modal = use_signal(|| false);
    let mut show_open_modal = use_signal(|| false);
    let mut show_share_modal = use_signal(|| false);
    let mut show_save_as_modal = use_signal(|| false);
    let mut save_status = use_signal(|| SaveStatus::Saved);

    // ui round trippers. set them manually when loaded_note changes
    let mut share_with = use_signal(move || String::new());
    let mut body = use_signal(move || String::new());
    let mut footnotes = use_signal(move || IndexMap::new());
    let mut loaded_note_full_path = use_signal(move || String::new());

    use_effect(move || {
        let full_path = file_path_segments().join("/");
        tracing::info!("loaded note full path changed to: {}", full_path);

        let Ok(note) = Note::from_path(PathBuf::from(&full_path), true) else {
            tracing::info!("note at {}  failed to load", full_path);
            // TODO: not sure what the error case represents
            return;
        };

        share_with.set(note.frontmatter.share_with.join(" "));
        body.set(note.content);
        footnotes.set(note.footnotes);
        loaded_note_full_path.set(full_path);
    });

    // derived from file_path_segments directly
    let relative_path = use_memo(move || {
        let loaded_from_path = PathBuf::from(file_path_segments().join("/"));
        app_context
            .vault
            .read()
            .absolute_path_to_relative_string(loaded_from_path)
    });
    let read_only = use_memo(move || relative_path.read().starts_with("footnotes"));

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

        footnotes_vec.retain(|name, _| link_names.contains(&name));
        for name in link_names {
            footnotes_vec
                .entry((&name).to_string())
                .or_insert(String::new());
        }

        tracing::info!("sync'd {} footnotes", footnotes_vec.len());
        footnotes_signal.set(footnotes_vec);
    };

    let save_link_to_footnote = move |(footnote_number, footnote_text)| async move {
        let mut footnotes_signal = footnotes.clone();
        let mut footnotes_vec = footnotes.read().clone();

        tracing::info!(
            "saving {} -> {} to footnotes",
            footnote_number,
            footnote_text
        );
        footnotes_vec.insert(footnote_number, footnote_text);
        footnotes_signal.set(footnotes_vec);
        save_status.set(SaveStatus::Unsaved);
    };

    let save_note = move |relative_path: Option<String>| async move {
        save_status.set(SaveStatus::Syncing);

        let full_path = file_path_segments().join("/");
        let mut note_copy = match Note::from_path(PathBuf::from(&full_path), true) {
            Ok(n) => n,
            Err(_) => Note::from_string("", true).unwrap(),
        };

        note_copy.frontmatter.share_with = share_with
            .read()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut footnotes_vec = footnotes.read().clone();
        footnotes_vec.retain(|_key, value| !value.trim().is_empty());

        let full_path = match relative_path {
            Some(p) => app_context
                .vault
                .read()
                .relative_string_to_absolute_path(&p),
            None => PathBuf::from(loaded_note_full_path()),
        };

        let mut note_body_eval =
            document::eval("dioxus.send(document.getElementById('note-body').value)");

        match note_body_eval.recv::<String>().await {
            Ok(note_body) => {
                tracing::info!("saving note to {}", full_path.to_string_lossy());
                note_copy.content = note_body;
                note_copy.footnotes = footnotes_vec;

                if let Err(e) = note_copy.to_file(&full_path) {
                    tracing::error!("Failed to save note: {e}");
                    save_status.set(SaveStatus::Unsaved);
                    return;
                }

                save_status.set(SaveStatus::Saved);
            }
            Err(e) => {
                tracing::error!("JavaScript Eval Error: {e:?}");
                save_status.set(SaveStatus::Unsaved);
            }
        }
    };

    let navigate_to_footnote = move |footnote_text: String| async move {
        tracing::info!("navigate_to_uuid: {}", footnote_text);

        if let Some(uuid_part) = footnote_text.split("footnote://").nth(1) {
            if let Ok(uuid) = Uuid::parse_str(&uuid_part) {
                if let Some(entry) = app_context.manifest.read().get(&uuid) {
                    tracing::info!(
                        "found entry for uuid, requesting nav to: {}",
                        entry.path.to_string_lossy()
                    );
                    nav.push(format!(
                        "/notes/{}",
                        &app_context
                            .vault
                            .read()
                            .base_path()
                            .join(&entry.path)
                            .to_string_lossy()
                            .to_string()
                    ));
                }
            }
        } else if footnote_text.starts_with("http://") || footnote_text.starts_with("https://") {
            tracing::info!("opening external link in system browser: {}", footnote_text);
            if let Err(e) = open::that(&footnote_text) {
                tracing::error!("failed to open link: {}", e);
            }
        }
    };

    let (status_icon, status_class) = match save_status() {
        SaveStatus::Saved => ("✓", "text-green-500"),
        SaveStatus::Unsaved => ("Save", "text-yellow-500"),
        SaveStatus::Syncing => ("↻", "text-blue-500 animate-spin"),
    };

    rsx! {
        AppHeader {
            on_menu_click: move |_| menu_visible.set(true),

            h1 {
                class: "flex-1 text-center text-sm font-medium text-zinc-300 truncate px-4",
                "{relative_path()}"
            }
            button {
                class: "w-8 text-center text-lg {status_class}",
                onclick: move |_| save_note(None),
                "{status_icon}"
            }
            SyncServiceToggle {}
        }

        AppMenu {
            visible: menu_visible(),
            on_close: move |_| menu_visible.set(false),


            MenuButton {
                label: "← Back",
                onclick: move |_| {
                    nav.go_back();
                    menu_visible.set(false);
                }
            }

            MenuButton {
                label: "→ Forward",
                onclick: move |_| {
                    nav.go_forward();
                    menu_visible.set(false);
                }
            }

            MenuButton {
                label: "New Note",
                onclick: move |_| {
                    show_new_note_modal.set(true);
                    menu_visible.set(false);
                }
            }

            MenuButton {
                label: "Open...",
                onclick: move |_| {
                    let mut app_context = use_context::<AppContext>();
                    if app_context.reload_manifest().is_ok() {
                        show_open_modal.set(true);
                    }
                    menu_visible.set(false);
                }
            }

            MenuButton {
                label: "Share...",
                onclick: move |_| {
                    show_share_modal.set(true);
                    menu_visible.set(false);
                }
            }

            MenuButton {
                label: "Save as...",
                onclick: move |_| {
                    show_save_as_modal.set(true);
                    menu_visible.set(false);
                }
            }
        }

        div {
            class: "flex-1 overflow-y-auto",

            div {
                class: "max-w-5xl mx-auto px-4 py-6 sm:px-6 space-y-6",
                textarea {
                    id: "note-body",
                    class: "w-full px-4 py-3 bg-zinc-900/30 border border-zinc-800 rounded-lg text-sm font-mono text-zinc-100 focus:border-zinc-700 focus:ring-1 focus:ring-zinc-700 focus:outline-none",
                    style: "min-height: max(60vh, 400px);",
                    onblur: sync_body_to_footnotes,
                    oninput: move |_| save_status.set(SaveStatus::Unsaved),
                    initial_value: "{body}",
                    readonly: read_only,
                }
            }

            div {
                class: "max-w-5xl pt-4 mx-auto px-4 pb-6 sm:px-6",
                Footnotes {
                    footnotes: footnotes,
                    onlink: save_link_to_footnote,
                    onlinkclick: navigate_to_footnote,
                }
            }
        }

        if show_new_note_modal() {
            NewNoteModal {
                ondone: move |_| show_new_note_modal.set(false)
            }
        }

        if show_open_modal() {
            NoteSelectModal {
                oncancel: move |_| show_open_modal.set(false),
                onselect: move |_| show_open_modal.set(false)
            }
        }

        if show_share_modal() {
            ShareWithModal {
                current_shares: share_with(),
                oncancel: move |_| show_share_modal.set(false),
                onsave: move |new_shares: String| {
                    share_with.set(new_shares);
                    show_share_modal.set(false);
                    save_status.set(SaveStatus::Unsaved);
                }
            }
        }

        if show_save_as_modal() {
            SaveAsModal {
                relative_path: relative_path.read(),
                oncancel: move |_| show_share_modal.set(false),
                onsave: move |new_relative_path: String| {
                    spawn(async move {
                        save_note(Some(new_relative_path.clone())).await;
                        show_save_as_modal.set(false);
                        save_status.set(SaveStatus::Saved);
                        nav.replace(format!(
                            "/notes/{}",
                            app_context.vault.read().relative_string_to_absolute_string(&new_relative_path)
                        ));
                    });
                }
            }
        }
    }
}

#[component]
fn NoteSelectModal(onselect: EventHandler, oncancel: EventHandler) -> Element {
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
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-2xl h-[80vh] border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl flex flex-col",
                onclick: move |evt| evt.stop_propagation(),

                div {
                    class: "sticky top-0 py-3 px-4 border-b bg-zinc-900 border-zinc-800 flex justify-between items-center",
                    h3 { class: "text-sm font-semibold", "Browse Files" }
                    button {
                        class: "p-1 rounded transition-colors text-zinc-500 hover:text-zinc-300",
                        onclick: move |_| oncancel.call(()),
                        "✕"
                    }
                }

                div {
                    class: "flex-1 overflow-y-auto p-2",
                    for (name, child) in root_children {
                        TreeNodeView {
                            name: name,
                            node: child,
                            onselect: onselect,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ShareWithModal(
    current_shares: String,
    oncancel: EventHandler,
    onsave: EventHandler<String>,
) -> Element {
    let mut share_input = use_signal(|| current_shares);

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-md border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl p-6",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4", "Share with contacts" }

                input {
                    class: "w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-sm focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 mb-4",
                    r#type: "text",
                    placeholder: "contact1 contact2",
                    value: "{share_input}",
                    oninput: move |e| share_input.set(e.value())
                }

                div { class: "flex gap-2 justify-end",
                    button {
                        class: "px-4 py-2 text-sm text-zinc-400 hover:text-zinc-100",
                        onclick: move |_| oncancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-zinc-100 text-zinc-900 rounded-md text-sm font-medium hover:bg-white",
                        onclick: move |_| onsave.call(share_input()),
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn SaveAsModal(
    relative_path: String,
    oncancel: EventHandler,
    onsave: EventHandler<String>,
) -> Element {
    let mut path_input = use_signal(|| relative_path);

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-md border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl p-6",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4", "Save as..." }

                input {
                    class: "w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-sm focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 mb-4",
                    r#type: "text",
                    value: "{path_input}",
                    oninput: move |e| path_input.set(e.value())
                }

                div { class: "flex gap-2 justify-end",
                    button {
                        class: "px-4 py-2 text-sm text-zinc-400 hover:text-zinc-100",
                        onclick: move |_| oncancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-zinc-100 text-zinc-900 rounded-md text-sm font-medium hover:bg-white",
                        onclick: move |_| onsave.call(path_input()),
                        "Save"
                    }
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
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
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
            button {
                class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick: toggle_open,
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
            div { class: "ml-6", {children} }
        } else {
            button {
                class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick: toggle_open,
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
            nav.push(format!(
                "/notes/{}",
                app_context
                    .vault
                    .read()
                    .base_path()
                    .join(relative_path)
                    .to_string_lossy()
                    .to_string()
            ));
            onselect(());
        }
    };

    rsx! {
        button {
            class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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

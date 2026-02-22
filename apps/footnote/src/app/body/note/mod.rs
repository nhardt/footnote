mod footnote_editor;
mod footnotes;
mod share_dropdown;

use dioxus::prelude::*;

use indexmap::IndexMap;
use regex::bytes::Regex;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use footnote_core::model::note::Note;
use footnote_core::util::lamport_timestamp::LamportTimestamp;
use footnote_core::util::manifest::find_responses;
use footnote_core::util::tombstone::tombstone_create;

use crate::body::note::footnotes::Footnotes;
use crate::body::note::share_dropdown::ShareDropdown;
use crate::context::{AppContext, MenuContext};
use crate::modal::confirm_modal::ConfirmModal;
#[derive(Clone, Copy, PartialEq)]
enum SaveStatus {
    Saved,
    Unsaved,
    Syncing,
}

#[component]
pub fn NoteView(vault_relative_path_segments: ReadSignal<Vec<String>>) -> Element {
    let nav = navigator();
    let mut app_context = use_context::<AppContext>();

    let mut show_share_modal = use_signal(|| false);
    let mut show_save_as_modal = use_signal(|| false);
    let mut show_delete_note_modal = use_signal(|| false);
    let mut delete_note_error = use_signal(|| String::new());
    let mut save_status = use_signal(|| SaveStatus::Saved);

    // ui round trippers. set them manually when loaded_note changes
    let mut share_with = use_signal(move || String::new());
    let mut footnotes = use_signal(move || IndexMap::new());
    let mut loaded_note_full_path = use_signal(move || String::new());
    let mut loaded_note_uuid = use_signal(move || Option::<Uuid>::None);
    let mut loaded_note_timestamp = use_signal(move || Option::<LamportTimestamp>::None);
    // responses: Vec<(relative_path, content)> for inline display (author view)
    let mut responses = use_signal(|| Vec::<(String, String)>::new());
    // responder state: editable response textarea
    let mut show_response_editor = use_signal(|| false);
    let mut response_body = use_signal(|| String::new());
    let mut response_save_status = use_signal(|| SaveStatus::Saved);

    let relative_path = use_memo(move || vault_relative_path_segments().join("/"));
    let read_only = use_memo(move || relative_path.read().starts_with("footnotes"));

    use_effect(move || {
        let vault_path = app_context.vault.read().base_path();
        loaded_note_uuid.set(None);
        loaded_note_timestamp.set(None);
        let full_path = vault_relative_path_segments()
            .iter()
            .fold(vault_path.clone(), |acc, seg| acc.join(seg));

        tracing::info!("loaded note full path changed to: {}", full_path.display());

        let Ok(note) = Note::from_path(&full_path, true) else {
            tracing::info!(
                "note failed to load from {}",
                full_path.to_string_lossy().to_string()
            );
            return;
        };

        share_with.set(note.frontmatter.share_with.join(" "));
        footnotes.set(note.footnotes);

        let note_uuid = note.frontmatter.uuid;
        let body_content = note.content.clone();
        spawn(async move {
            let json_content = serde_json::to_string(&body_content).unwrap_or_default();
            let _ = document::eval(&format!(
                r#"document.getElementById("note-body").value = {};"#,
                json_content
            ))
            .await;
        });

        // check for existing response file (responder view)
        let reply_path = vault_path.join(format!("_replies/response-to-{}.md", note_uuid));
        if reply_path.exists() {
            if let Ok(reply_note) = Note::from_path(&reply_path, true) {
                response_body.set(reply_note.content);
                show_response_editor.set(true);
            }
        } else {
            response_body.set(String::new());
            show_response_editor.set(false);
        }
        response_save_status.set(SaveStatus::Saved);

        // find all responses for inline display (author view)
        spawn(async move {
            let vp = vault_path.clone();
            match tokio::task::spawn_blocking(move || {
                let entries = find_responses(&vp, note_uuid)?;
                let mut result = Vec::new();
                for entry in entries {
                    let abs = vp.join(&entry.path);
                    if let Ok(n) = Note::from_path(&abs, false) {
                        result.push((entry.path.to_string_lossy().to_string(), n.content));
                    }
                }
                Ok::<_, anyhow::Error>(result)
            })
            .await
            {
                Ok(Ok(found)) => responses.set(found),
                Ok(Err(e)) => tracing::warn!("find_responses error: {e}"),
                Err(e) => tracing::warn!("find_responses task failed: {e}"),
            }
        });

        loaded_note_full_path.set(full_path.to_string_lossy().to_string());
        loaded_note_uuid.set(Some(note_uuid));
        loaded_note_timestamp.set(Some(note.frontmatter.modified));
    });

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

    let delete_note_confirm = move |_| {
        spawn(async move {
            if let Some(note_uuid) = loaded_note_uuid() {
                tracing::info!("deleting {}", note_uuid);
                let result = tombstone_create(
                    &app_context.vault.read().base_path(),
                    note_uuid,
                    LamportTimestamp::new(loaded_note_timestamp()),
                )
                .await;
                if let Err(e) = result {
                    delete_note_error.set(format!("{}", e));
                    return;
                }
                if let Err(e) = fs::remove_file(PathBuf::from(loaded_note_full_path())) {
                    delete_note_error.set(format!("{}", e));
                    return;
                }
                if let Err(e) = app_context.reload_manifest() {
                    tracing::warn!("failed to reload manifest: {}", e);
                }
                consume_context::<MenuContext>().go_home();
            }
        });
    };

    let save_note = move |relative_path: Option<String>| async move {
        save_status.set(SaveStatus::Syncing);

        let mut note_copy = match Note::from_path(PathBuf::from(loaded_note_full_path()), true) {
            Ok(n) => n,
            Err(e) => {
                tracing::info!("failed to load note for RMW, {}", e);
                Note::new()
            }
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

    // Semantics for navigating:
    // - if footnote_text is http(s) link, do external nav
    // - if footnote_text is footnote link with uuid, do internal nav
    // - if source page for link is under footnotes/user and page exists under
    //     footnotes/user, navigate
    // - if source page for link is under footnotes/user and page does not exist, log and return
    // - if vault dir + footnote_text resolves to path under vault dir and vault
    //     + footnote_text exists, navigate to it
    // - if vault dir + footnote_text resolves to path under vault dir and vault
    //     footnote_text does not exist, create and navigate to it
    let navigate_to_footnote = move |footnote_text: String| async move {
        tracing::info!("navigate_to_uuid: {}", footnote_text);

        let result: Result<()> = async {
            if footnote_text.starts_with("http://") || footnote_text.starts_with("https://") {
                tracing::info!("opening external link in system browser: {}", footnote_text);
                open::that(&footnote_text).context("failed to open external link")?;
                return Ok(());
            } else if let Some(uuid_part) = footnote_text.split("footnote://").nth(1) {
                if let Ok(uuid) = Uuid::parse_str(&uuid_part) {
                    if let Some(entry) = app_context.manifest.read().get(&uuid) {
                        tracing::info!(
                            "found entry for uuid, requesting nav to: {}",
                            entry.path.to_string_lossy()
                        );
                        consume_context::<MenuContext>()
                            .go_note(&entry.path.to_string_lossy().to_string());
                        return Ok(());
                    }
                }
            } else {
                let vault_base = app_context.vault.read().base_path();
                let canonical_footnotes = vault_base.join("footnotes").canonicalize()?;
                let current_note_path = loaded_note_full_path()
                    .as_str()
                    .parse::<PathBuf>()
                    .context("invalid path")?
                    .canonicalize()?;

                if let Ok(relative) = current_note_path.strip_prefix(&canonical_footnotes) {
                    if let Some(remote_user_dir) = relative.components().next() {
                        tracing::info!(
                            "current path is in {}, will see if relative path exists",
                            remote_user_dir.as_os_str().to_string_lossy()
                        );
                        // since we clicked:
                        // - a link from a document that is not ours,
                        // - the link is not an known url
                        // - it is not a link by uuid
                        // we will see if the relative path exists
                        let relative_path = canonical_footnotes
                            .join(remote_user_dir.as_os_str())
                            .join(&footnote_text);

                        if relative_path.exists() {
                            consume_context::<MenuContext>()
                                .go_note(&relative_path.to_string_lossy().to_string());
                            return Ok(());
                        } else {
                            // if, not, this is just a link in a doc we don't have
                            tracing::info!(
                                "link from footnotes/ to non-existent file, not creating: {}",
                                footnote_text
                            );
                            return Ok(());
                        }
                    }
                }

                let path = PathBuf::from(&footnote_text);
                let full_path = vault_base.join(&path);
                let canonical_base = vault_base.canonicalize()?;

                let canonical_full = if full_path.exists() {
                    full_path.canonicalize()?
                } else {
                    let parent = full_path.parent().context("no parent")?;
                    fs::create_dir_all(parent)?;
                    parent
                        .canonicalize()?
                        .join(full_path.file_name().context("no filename")?)
                };

                if !canonical_full.starts_with(&canonical_base) {
                    tracing::info!(
                        "this is not a local link, will not create file: {}",
                        footnote_text
                    );
                    return Ok(());
                }

                // Create file if needed, then navigate
                if !canonical_full.exists() {
                    fs::write(&canonical_full, "")?;
                }

                consume_context::<MenuContext>().go_note(&footnote_text);
            }

            Ok(())
        }
        .await;

        if let Err(e) = result {
            tracing::error!("navigation failed: {}", e);
        }
    };

    let (status_icon, status_class) = match save_status() {
        SaveStatus::Saved => ("Saved", "text-green-500"),
        SaveStatus::Unsaved => ("Save", "text-yellow-500"),
        SaveStatus::Syncing => ("Saving...", "text-blue-500 animate-spin"),
    };

    rsx! {
        div {
            class: "flex-1 overflow-y-auto",
            div {
                class: "max-w-3xl mx-auto px-6 py-6",

                div {
                    class: "text-xs font-mono text-zinc-500 mb-4",
                    "{relative_path()}"
                }

                textarea {
                    id: "note-body",
                    class: "w-full px-4 py-4 bg-zinc-900/30 border border-zinc-800 rounded-lg text-sm font-mono text-zinc-100 focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 focus:outline-none mb-6",
                    style: "min-height: max(60vh, 400px);",
                    onblur: sync_body_to_footnotes,
                    oninput: move |_| save_status.set(SaveStatus::Unsaved),
                    readonly: read_only,
                }

                if !read_only() {
                    div {
                        class: "flex items-center justify-end gap-2 mt-2 mb-6",
                        button {
                            class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                            onclick: move |_| show_delete_note_modal.set(true),
                            "Delete"
                        }
                        if !app_context.contacts.read().is_empty() {
                            ShareDropdown {
                                share_with: share_with,
                                on_change: move |_| save_status.set(SaveStatus::Unsaved),
                            }
                        }
                        button {
                            class: "px-3 py-1.5 text-sm font-medium rounded-md transition-colors {status_class}",
                            onclick: move |_| save_note(None),
                            "{status_icon}"
                        }
                    }
                }

                div {
                    class: "mb-6 border border-zinc-800 rounded-lg bg-zinc-900/30",
                    Footnotes {
                        footnotes: footnotes,
                        onlink: save_link_to_footnote,
                        onlinkclick: navigate_to_footnote,
                    }
                }

                div {
                    class: "border border-zinc-800 rounded-lg bg-zinc-900/30",
                    div {
                        class: "px-4 py-3 border-b border-zinc-800 flex items-center justify-between",
                        h2 {
                            class: "text-sm font-semibold font-mono text-zinc-400",
                            "Responses"
                        }
                        if read_only() && !show_response_editor() {
                            button {
                                class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                                onclick: move |_| show_response_editor.set(true),
                                "Add Response"
                            }
                        }
                    }

                    // Responder view: editable response textarea
                    if read_only() && show_response_editor() {
                        div {
                            class: "px-4 py-4",
                            textarea {
                                class: "w-full px-4 py-4 bg-zinc-900/30 border border-zinc-800 rounded-lg text-sm font-mono text-zinc-100 focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 focus:outline-none mb-3",
                                style: "min-height: 200px;",
                                placeholder: "Write your response...",
                                value: "{response_body}",
                                oninput: move |e| {
                                    response_body.set(e.value());
                                    response_save_status.set(SaveStatus::Unsaved);
                                },
                            }
                            {
                                let (resp_icon, resp_class) = match response_save_status() {
                                    SaveStatus::Saved => ("✓", "text-green-500"),
                                    SaveStatus::Unsaved => ("Save Response", "text-yellow-500"),
                                    SaveStatus::Syncing => ("↻", "text-blue-500 animate-spin"),
                                };
                                rsx! {
                                    button {
                                        class: "px-3 py-1.5 text-sm font-medium hover:bg-zinc-800 rounded-md transition-colors {resp_class}",
                                        onclick: move |_| {
                                            let vault_path = app_context.vault.read().base_path();
                                            let rel = relative_path();
                                            let owner = rel
                                                .strip_prefix("footnotes/")
                                                .and_then(|rest| rest.split('/').next())
                                                .unwrap_or("")
                                                .to_string();

                                            let full_path = vault_relative_path_segments()
                                                .iter()
                                                .fold(vault_path.clone(), |acc, seg| acc.join(seg));
                                            let Ok(note) = Note::from_path(&full_path, true) else {
                                                return;
                                            };
                                            let target_uuid = note.frontmatter.uuid;
                                            let reply_path = vault_path.join(format!("_replies/response-to-{}.md", target_uuid));

                                            response_save_status.set(SaveStatus::Syncing);

                                            if let Some(parent) = reply_path.parent() {
                                                let _ = fs::create_dir_all(parent);
                                            }

                                            let mut reply_note = if reply_path.exists() {
                                                Note::from_path(&reply_path, true).unwrap_or_else(|_| Note::new())
                                            } else {
                                                let mut n = Note::new();
                                                n.frontmatter.reply_to = Some(target_uuid);
                                                if !owner.is_empty() {
                                                    n.frontmatter.share_with = vec![owner];
                                                }
                                                n
                                            };

                                            reply_note.content = response_body();
                                            match reply_note.to_file(&reply_path) {
                                                Ok(_) => response_save_status.set(SaveStatus::Saved),
                                                Err(e) => {
                                                    tracing::error!("failed to save response: {e}");
                                                    response_save_status.set(SaveStatus::Unsaved);
                                                }
                                            }
                                        },
                                        "{resp_icon}"
                                    }
                                }
                            }
                        }
                    }

                    if !read_only() {
                        if responses().is_empty() {
                            div {
                                class: "px-4 py-4 text-sm text-zinc-500",
                                "No responses yet"
                            }
                        } else {
                            for (path, content) in responses() {
                                div {
                                    key: "{path}",
                                    class: "border-b border-zinc-800/50",
                                    div {
                                        class: "px-4 py-2 text-xs font-mono text-zinc-500",
                                        "{path}"
                                    }
                                    pre {
                                        class: "px-4 py-3 text-sm font-mono text-zinc-300 whitespace-pre-wrap",
                                        "{content}"
                                    }
                                }
                            }
                        }
                    }
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
                            "/note/{}",
                            app_context.vault.read().relative_string_to_absolute_string(&new_relative_path)
                        ));
                    });
                }
            }
        }

        if show_delete_note_modal() {
            ConfirmModal {
                oncancel: move || show_delete_note_modal.set(false),
                onconfirm: delete_note_confirm,
                p { class: "text-sm text-zinc-300 mb-6",
                    "Deleting this note will delete the file locally
                    immediately. The delete will be synced to other nodes on
                    next connection. If the file is modified on a remote node
                    after this delete, the remote version will be kept."
                }
                if !delete_note_error().is_empty() {
                    div { class: "text-sm text-red-400", "{delete_note_error}" }
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

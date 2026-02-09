mod footnote_editor;
mod footnotes;

use dioxus::prelude::*;

use indexmap::IndexMap;
use regex::bytes::Regex;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use footnote_core::model::note::Note;

use crate::body::note::footnotes::Footnotes;
use crate::context::{AppContext, MenuContext};

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

    let mut show_share_modal = use_signal(|| false);
    let mut show_save_as_modal = use_signal(|| false);
    let mut save_status = use_signal(|| SaveStatus::Saved);

    // ui round trippers. set them manually when loaded_note changes
    let mut share_with = use_signal(move || String::new());
    let mut body = use_signal(move || String::new());
    let mut footnotes = use_signal(move || IndexMap::new());
    let mut loaded_note_full_path = use_signal(move || String::new());

    use_effect(move || {
        let vault_path = app_context.vault.read().base_path();
        let full_path = file_path_segments()
            .iter()
            .fold(vault_path, |acc, seg| acc.join(seg));

        tracing::info!("loaded note full path changed to: {}", full_path.display());

        let Ok(note) = Note::from_path(&full_path, true) else {
            tracing::info!("note failed to load");
            return;
        };

        share_with.set(note.frontmatter.share_with.join(" "));
        footnotes.set(note.footnotes);

        let body_content = note.content.clone();
        spawn(async move {
            let json_content = serde_json::to_string(&body_content).unwrap_or_default();
            let _ = document::eval(&format!(
                r#"document.getElementById("note-body").value = {};"#,
                json_content
            ))
            .await;
        });

        loaded_note_full_path.set(full_path.to_string_lossy().to_string());
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
        SaveStatus::Saved => ("✓", "text-green-500"),
        SaveStatus::Unsaved => ("Save", "text-yellow-500"),
        SaveStatus::Syncing => ("↻", "text-blue-500 animate-spin"),
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

                if !read_only() {
                    div {
                        class: "flex items-center gap-2 mb-4",
                        button {
                            class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors flex items-center gap-2 {status_class}",
                            onclick: move |_| save_note(None),
                            "{status_icon}"
                        }
                        button {
                            class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                            onclick: move |_| show_share_modal.set(true),
                            "Share"
                        }
                    }
                }

                textarea {
                    id: "note-body",
                    class: "w-full px-4 py-4 bg-zinc-900/30 border border-zinc-800 rounded-lg text-sm font-mono text-zinc-100 focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 focus:outline-none mb-6",
                    style: "min-height: max(60vh, 400px);",
                    onblur: sync_body_to_footnotes,
                    oninput: move |_| save_status.set(SaveStatus::Unsaved),
                    readonly: read_only,
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
                        if read_only() {
                            button {
                                class: "px-3 py-1.5 text-sm font-medium text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-md transition-colors",
                                "Add Response"
                            }
                        }
                    }
                    div {
                        class: "px-4 py-4 text-sm text-zinc-500",
                        "No responses yet"
                    }
                }
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
                            "/note/{}",
                            app_context.vault.read().relative_string_to_absolute_string(&new_relative_path)
                        ));
                    });
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

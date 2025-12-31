
    html { class: "dark", lang: "en",
        head {
            meta { charset: "UTF-8" }
            meta {
                content: "width=device-width, initial-scale=1.0",
                name: "viewport",
            }
            title { "Footnote.wiki - File Browser" }
            script { src: "https://cdn.tailwindcss.com" }
            link { href: "https://fonts.googleapis.com", rel: "preconnect" }
            link {
                crossorigin: "false",
                href: "https://fonts.gstatic.com",
                rel: "preconnect",
            }
            link {
                href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=IBM+Plex+Sans:wght@400;500;600&display=swap",
                rel: "stylesheet",
            }
            script {
                "tailwind.config = {\n                darkMode: \"class\",\n                theme: {\n                    extend: {\n                        fontFamily: {\n                            sans: [\"IBM Plex Sans\", \"system-ui\", \"sans-serif\"],\n                            mono: [\"JetBrains Mono\", \"monospace\"],\n                        },\n                    },\n                },\n            };"
            }
            style {
                "* {\n                transition:\n                    border-color 150ms ease,\n                    background-color 150ms ease,\n                    box-shadow 150ms ease;\n            }\n\n            input:focus,\n            textarea:focus,\n            button:focus-visible {\n                outline: none;\n            }"
            }
        }
        body { class: "bg-zinc-950 text-zinc-100 font-sans antialiased",
            div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
                div {
                    class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full max-h-[90vh] flex flex-col",
                    "onclick": "event.stopPropagation()",
                    div { class: "p-6 border-b border-zinc-800",
                        h2 { class: "text-lg font-semibold font-mono", "Select Note" }
                        p { class: "text-sm text-zinc-500 mt-1",
                            "\n                        Browse your vault and select a file\n                    "
                        }
                    }
                    div { class: "p-6 border-b border-zinc-800 bg-zinc-900/50",
                        div { class: "flex items-center gap-2 mb-4",
                            div { class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-md text-sm font-mono text-zinc-300 truncate",
                                "\n                            /home/nhardt/footnote.wiki\n                        "
                            }
                            button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                                svg {
                                    class: "w-4 h-4",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M5 15l7-7 7 7",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                    }
                                }
                                "\n                            Up\n                        "
                            }
                        }
                        div { class: "flex gap-2",
                            button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                                svg {
                                    class: "w-4 h-4",
                                    fill: "currentColor",
                                    view_box: "0 0 20 20",
                                    path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                                }
                                "\n                            Folder\n                        "
                            }
                            button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                                svg {
                                    class: "w-4 h-4",
                                    fill: "currentColor",
                                    view_box: "0 0 20 20",
                                    path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                                }
                                "\n                            File\n                        "
                            }
                        }
                    }
                    div { class: "flex-1 overflow-y-auto min-h-0 p-6",
                        div { class: "mb-6",
                            h3 { class: "text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-3",
                                "\n                            Folders\n                        "
                            }
                            div { class: "space-y-1",
                                button { class: "w-full flex items-center gap-3 px-3 py-2 hover:bg-zinc-800/50 rounded-md text-left transition-colors group",
                                    svg {
                                        class: "w-5 h-5 text-zinc-500 group-hover:text-zinc-400",
                                        fill: "currentColor",
                                        view_box: "0 0 20 20",
                                        path { d: "M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" }
                                    }
                                    span { class: "text-sm font-medium", "footnotes" }
                                }
                            }
                        }
                        div {
                            h3 { class: "text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-3",
                                "\n                            Files\n                        "
                            }
                            div { class: "space-y-1",
                                button { class: "w-full flex items-center gap-3 px-3 py-2 hover:bg-zinc-800/50 rounded-md text-left transition-colors group",
                                    svg {
                                        class: "w-5 h-5 text-zinc-500 group-hover:text-zinc-400",
                                        fill: "currentColor",
                                        view_box: "0 0 20 20",
                                        path {
                                            clip_rule: "evenodd",
                                            d: "M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z",
                                            fill_rule: "evenodd",
                                        }
                                    }
                                    span { class: "text-sm", "home.md" }
                                }
                                button { class: "w-full flex items-center gap-3 px-3 py-2 hover:bg-zinc-800/50 rounded-md text-left transition-colors group",
                                    svg {
                                        class: "w-5 h-5 text-zinc-500 group-hover:text-zinc-400",
                                        fill: "currentColor",
                                        view_box: "0 0 20 20",
                                        path {
                                            clip_rule: "evenodd",
                                            d: "M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z",
                                            fill_rule: "evenodd",
                                        }
                                    }
                                    span { class: "text-sm", "note-apple.md" }
                                }
                                button { class: "w-full flex items-center gap-3 px-3 py-2 hover:bg-zinc-800/50 rounded-md text-left transition-colors group",
                                    svg {
                                        class: "w-5 h-5 text-zinc-500 group-hover:text-zinc-400",
                                        fill: "currentColor",
                                        view_box: "0 0 20 20",
                                        path {
                                            clip_rule: "evenodd",
                                            d: "M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z",
                                            fill_rule: "evenodd",
                                        }
                                    }
                                    span { class: "text-sm", "note-linux.md" }
                                }
                            }
                        }
                    }
                    div { class: "p-6 border-t border-zinc-800 bg-zinc-900/50",
                        div { class: "flex gap-3",
                            button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                "\n                            Cancel\n                        "
                            }
                            button { class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                "\n                            Select Note\n                        "
                            }
                        }
                    }
                }
            }
            div { id: "new-folder-state", style: "display: none",
                div { class: "p-6 border-b border-zinc-800 bg-zinc-900/50",
                    div { class: "flex items-center gap-2 mb-4",
                        div { class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-md text-sm font-mono text-zinc-300 truncate",
                            "\n                        /home/nhardt/footnote.wiki\n                    "
                        }
                        button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M5 15l7-7 7 7",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }
                            "\n                        Up\n                    "
                        }
                    }
                    div { class: "bg-zinc-800/30 border border-zinc-700 rounded-lg p-4",
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "New Folder Name"
                        }
                        div { class: "flex gap-2",
                            input {
                                autofocus: "false",
                                class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                placeholder: "folder-name",
                                r#type: "text",
                            }
                            button { class: "px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                "\n                            Create\n                        "
                            }
                            button { class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                "\n                            Cancel\n                        "
                            }
                        }
                    }
                }
            }
            div { id: "new-file-state", style: "display: none",
                div { class: "p-6 border-b border-zinc-800 bg-zinc-900/50",
                    div { class: "flex items-center gap-2 mb-4",
                        div { class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-md text-sm font-mono text-zinc-300 truncate",
                            "\n                        /home/nhardt/footnote.wiki\n                    "
                        }
                        button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M5 15l7-7 7 7",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }
                            "\n                        Up\n                    "
                        }
                    }
                    div { class: "bg-zinc-800/30 border border-zinc-700 rounded-lg p-4",
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "New File Name"
                        }
                        div { class: "flex gap-2",
                            input {
                                autofocus: "false",
                                class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                placeholder: "file-name.md",
                                r#type: "text",
                            }
                            button { class: "px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                "\n                            Create\n                        "
                            }
                            button { class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                "\n                            Cancel\n                        "
                            }
                        }
                    }
                }
            }
        }
    }

    html { class: "dark", lang: "en",
        head {
            meta { charset: "UTF-8" }
            meta {
                content: "width=device-width, initial-scale=1.0",
                name: "viewport",
            }
            title { "Footnote.wiki - Note Editor" }
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
                "* {\n                transition:\n                    border-color 150ms ease,\n                    background-color 150ms ease,\n                    box-shadow 150ms ease;\n            }\n\n            input:focus,\n            textarea:focus,\n            button:focus-visible {\n                outline: none;\n            }\n\n            /* Ensure full height */\n            html,\n            body {\n                height: 100%;\n                margin: 0;\n            }"
            }
        }
        body { class: "bg-zinc-950 text-zinc-100 font-sans antialiased",
            nav { class: "border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-sm",
                div { class: "px-6 py-3",
                    div { class: "flex items-center gap-8",
                        button { class: "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100",
                            "\n                        Notes\n                    "
                        }
                        button { class: "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors",
                            "\n                        Profile\n                    "
                        }
                        button { class: "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors",
                            "\n                        Contacts\n                    "
                        }
                        div { class: "ml-auto flex items-center gap-2",
                            div { class: "h-2 w-2 rounded-full bg-zinc-500" }
                            span { class: "text-sm font-mono text-zinc-400", "Sync Inactive" }
                        }
                    }
                }
            }
            div { class: "flex flex-col h-[calc(100vh-57px)]",
                div { class: "border-b border-zinc-800 bg-zinc-900/30 px-6 py-4",
                    div { class: "max-w-5xl mx-auto",
                        div { class: "grid grid-cols-[auto_1fr] gap-x-4 gap-y-3 items-center",
                            label { class: "text-sm font-medium text-zinc-400", "File" }
                            input {
                                class: "px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                r#type: "text",
                                value: "home.md",
                            }
                            label { class: "text-sm font-medium text-zinc-400", "Shared with" }
                            div { class: "flex items-center gap-2",
                                input {
                                    class: "flex-1 px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                    placeholder: "alice bob charlie",
                                    r#type: "text",
                                }
                                button { class: "px-4 py-1.5 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                    "\n                                Save\n                            "
                                }
                            }
                        }
                    }
                }
                div { class: "flex-1 overflow-hidden",
                    div { class: "h-full max-w-5xl mx-auto px-6 py-6",
                        textarea {
                            class: "w-full h-full px-4 py-3 bg-zinc-900/30 border border-zinc-800 rounded-lg text-sm font-mono text-zinc-100 resize-none focus:border-zinc-700 focus:ring-1 focus:ring-zinc-700",
                            placeholder: "Start writing...",
                            "Welcome to footnote.wiki\n\nThis is your personal knowledge base. Write in markdown, link notes together, and sync across your devices.\n\n## Getting Started\n\nCreate new notes using the browser, or start typing here.\n\n## Features\n\n- Local-first storage\n- P2P sync across your devices\n- Share with trusted contacts\n- Markdown formatting\n- [[Wiki-style links]]"
                        }
                    }
                }
            }
        }
    }

    html { class: "dark", lang: "en",
        head {
            meta { charset: "UTF-8" }
            meta {
                content: "width=device-width, initial-scale=1.0",
                name: "viewport",
            }
            title { "Footnote.wiki - Profile" }
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
                "tailwind.config = {\n                darkMode: \"class\",\n                theme: {\n                    extend: {\n                        fontFamily: {\n                            sans: [\"IBM Plex Sans\", \"system-ui\", \"sans-serif\"],\n                            mono: [\"JetBrains Mono\", \"monospace\"],\n                        },\n                        colors: {\n                            border: \"rgb(39 39 42)\", // zinc-800\n                        },\n                    },\n                },\n            };"
            }
            style {
                "/* Smooth animations */\n            * {\n                transition:\n                    border-color 150ms ease,\n                    background-color 150ms ease,\n                    box-shadow 150ms ease;\n            }\n\n            /* Custom focus ring */\n            input:focus,\n            textarea:focus,\n            button:focus-visible {\n                outline: none;\n            }"
            }
        }
        body { class: "bg-zinc-950 text-zinc-100 font-sans antialiased",
            nav { class: "border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-sm",
                div { class: "px-6 py-3",
                    div { class: "flex items-center gap-8",
                        button { class: "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors",
                            "\n                        Notes\n                    "
                        }
                        button { class: "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100",
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
            main { class: "max-w-3xl mx-auto px-6 py-12",
                div { class: "mb-12",
                    h1 { class: "text-3xl font-bold font-mono mb-2",
                        "\n                    Profile: "
                        span { class: "text-zinc-400", "Primary" }
                    }
                    p { class: "text-sm text-zinc-500",
                        "\n                    Manage your vault identity and connected devices\n                "
                    }
                }
                div { class: "space-y-8",
                    section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-6",
                        div { class: "flex items-center gap-4",
                            label { class: "text-sm font-medium text-zinc-300 w-32", "Username" }
                            input {
                                class: "flex-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                r#type: "text",
                                value: "nateh",
                            }
                            button { class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                "\n                            Update\n                        "
                            }
                        }
                    }
                    section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                        div { class: "p-6 border-b border-zinc-800",
                            h2 { class: "text-lg font-semibold font-mono", "Devices" }
                            p { class: "text-sm text-zinc-500 mt-1",
                                "\n                            Your connected devices in this vault\n                        "
                            }
                        }
                        div { class: "divide-y divide-zinc-800",
                            div { class: "p-6 hover:bg-zinc-900/50 transition-colors group",
                                div { class: "flex items-start justify-between",
                                    div { class: "flex-1 min-w-0",
                                        div { class: "flex items-center gap-3 mb-2",
                                            h3 { class: "text-sm font-semibold",
                                                "\n                                            default_device_name\n                                        "
                                            }
                                            span { class: "px-2 py-0.5 bg-zinc-800 border border-zinc-700 rounded text-xs font-mono text-zinc-400",
                                                "\n                                            Primary\n                                        "
                                            }
                                        }
                                        p { class: "text-xs font-mono text-zinc-500 truncate",
                                            "\n                                        07b87ce9aa468bc8af9627e581d305ca2722...\n                                    "
                                        }
                                    }
                                }
                            }
                            div { class: "p-6 hover:bg-zinc-900/50 transition-colors group",
                                div { class: "flex items-start justify-between",
                                    div { class: "flex-1 min-w-0",
                                        div { class: "flex items-center gap-3 mb-2",
                                            h3 { class: "text-sm font-semibold",
                                                "\n                                            laptop\n                                        "
                                            }
                                        }
                                        p { class: "text-xs font-mono text-zinc-500 truncate",
                                            "\n                                        121bae4696cd827d18c431547f343083a43...\n                                    "
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "p-6 bg-zinc-900/20 border-t border-zinc-800",
                            button { class: "flex items-center gap-3 text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-colors group",
                                div { class: "p-1.5 rounded-full bg-zinc-800 group-hover:bg-zinc-700 border border-zinc-700 group-hover:border-zinc-600 transition-all",
                                    svg {
                                        class: "w-4 h-4",
                                        fill: "currentColor",
                                        view_box: "0 0 20 20",
                                        path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                                    }
                                }
                                span { "Join a device you own to this vault" }
                            }
                        }
                    }
                    section {
                        button { class: "w-full px-6 py-4 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 rounded-lg text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-all text-left",
                            div { class: "flex items-center justify-between",
                                div {
                                    div { class: "font-semibold mb-1",
                                        "\n                                    Create Contact Record\n                                "
                                    }
                                    div { class: "text-xs text-zinc-500",
                                        "\n                                    Share with your trust network\n                                "
                                    }
                                }
                                svg {
                                    class: "w-5 h-5 text-zinc-500",
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
                            }
                        }
                    }
                }
            }
            div { id: "standalone-state", style: "display: none",
                main { class: "max-w-3xl mx-auto px-6 py-12",
                    div { class: "mb-12",
                        h1 { class: "text-3xl font-bold font-mono mb-2",
                            "\n                        Profile: "
                            span { class: "text-zinc-400", "Standalone" }
                        }
                        p { class: "text-sm text-zinc-500",
                            "\n                        Your vault is ready for local use\n                    "
                        }
                    }
                    div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-8",
                        p { class: "text-zinc-300 mb-8",
                            "\n                        You're using footnote in standalone mode. Would you like\n                        to sync with other devices?\n                    "
                        }
                        div { class: "flex gap-4",
                            button { class: "flex-1 px-6 py-3 bg-zinc-100 hover:bg-white text-zinc-900 rounded-lg font-medium transition-all",
                                "\n                            Make This Primary\n                        "
                            }
                            button { class: "flex-1 px-6 py-3 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 text-zinc-100 rounded-lg font-medium transition-all",
                                "\n                            Join Existing Vault\n                        "
                            }
                        }
                    }
                }
            }
            div {
                class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
                id: "join-listen-modal",
                style: "display: none",
                div {
                    class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                    "onclick": "event.stopPropagation()",
                    div { class: "p-6 border-b border-zinc-800",
                        h3 { class: "text-lg font-semibold font-mono", "Join Device" }
                        p { class: "text-sm text-zinc-500 mt-1",
                            "\n                        Share this URL with your secondary device\n                    "
                        }
                    }
                    div { class: "p-6",
                        div { class: "bg-zinc-950 border border-zinc-800 rounded-lg p-4 mb-6",
                            p { class: "text-xs font-mono text-zinc-400 break-all",
                                "\n                            iroh://abc123def456...\n                        "
                            }
                        }
                        div { class: "flex gap-3",
                            button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                "\n                            Copy URL\n                        "
                            }
                            button { class: "px-4 py-2 bg-zinc-100 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                                "\n                            Done\n                        "
                            }
                        }
                    }
                }
            }
            div {
                class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
                id: "join-modal",
                style: "display: none",
                div {
                    class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                    "onclick": "event.stopPropagation()",
                    div { class: "p-6 border-b border-zinc-800",
                        h3 { class: "text-lg font-semibold font-mono",
                            "\n                        Join Listening Device\n                    "
                        }
                        p { class: "text-sm text-zinc-500 mt-1",
                            "\n                        Enter the join URL from your primary device\n                    "
                        }
                    }
                    div { class: "p-6",
                        div { class: "mb-6",
                            label { class: "block text-sm font-medium text-zinc-300 mb-2",
                                "Join URL"
                            }
                            input {
                                class: "w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-md text-sm font-mono focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600",
                                placeholder: "iroh://...",
                                r#type: "text",
                            }
                        }
                        div { class: "flex gap-3",
                            button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                "\n                            Cancel\n                        "
                            }
                            button { class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                                "\n                            Join\n                        "
                            }
                        }
                    }
                }
            }
            div {
                class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
                id: "export-modal",
                style: "display: none",
                div {
                    class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full max-h-[90vh] flex flex-col",
                    "onclick": "event.stopPropagation()",
                    div { class: "p-6 border-b border-zinc-800",
                        h3 { class: "text-lg font-semibold font-mono",
                            "\n                        Export Contact Record\n                    "
                        }
                        p { class: "text-sm text-zinc-500 mt-1",
                            "\n                        Copy and share this with your trusted contacts\n                    "
                        }
                    }
                    div { class: "p-6 flex-1 min-h-0 flex flex-col",
                        textarea {
                            class: "flex-1 w-full px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-300 resize-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 mb-4",
                            readonly: "false",
                            "{\n  \"nickname\": \"@nateh\",\n  \"username\": \"nateh\",\n  \"master_public_key\": \"abc123...\",\n  \"primary_device_name\": \"laptop\",\n  \"devices\": [\n    {\n      \"name\": \"laptop\",\n      \"iroh_endpoint_id\": \"121bae4696...\",\n      \"timestamp\": \"2024-01-15T10:30:00Z\"\n    }\n  ],\n  \"timestamp\": \"2024-01-15T10:30:00Z\",\n  \"signature\": \"def456...\"\n}"
                        }
                        button { class: "w-full px-4 py-2 bg-zinc-100 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                            "\n                        Done\n                    "
                        }
                    }
                }
            }
        }
    }
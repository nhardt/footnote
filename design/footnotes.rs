
    html { lang: "en",
        head {
            meta { charset: "UTF-8" }
            meta {
                content: "width=device-width, initial-scale=1.0",
                name: "viewport",
            }
            title { "Footnotes Display Component" }
            script { src: "https://cdn.tailwindcss.com" }
        }
        body { class: "p-8 bg-zinc-950 text-zinc-100",
            div { class: "mx-auto space-y-8 max-w-5xl",
                div { class: "overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900/30",
                    div { class: "py-3 px-4 border-b border-zinc-800 bg-zinc-900/50",
                        h3 { class: "text-sm font-semibold text-zinc-300",
                            "\n                        Footnotes\n                    "
                        }
                    }
                    div { class: "divide-y divide-zinc-800",
                        div { class: "py-3 px-4 transition-colors group hover:bg-zinc-800/50",
                            div { class: "flex gap-3 items-center",
                                span { class: "flex-shrink-0 w-20 font-mono text-xs text-zinc-500",
                                    "[^intro]"
                                }
                                button { class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-300 hover:text-zinc-100",
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
                                    span { "Introduction to Distributed Systems" }
                                }
                                button { class: "p-1.5 rounded opacity-0 transition-all group-hover:opacity-100 text-zinc-500 hover:text-zinc-300 hover:bg-zinc-700",
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
                        div { class: "py-3 px-4 transition-colors group hover:bg-zinc-800/50",
                            div { class: "flex gap-3 items-center",
                                span { class: "flex-shrink-0 w-20 font-mono text-xs text-zinc-500",
                                    "[^source2]"
                                }
                                button { class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-300 hover:text-zinc-100",
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
                                    span { "My Research Notes" }
                                }
                                button { class: "p-1.5 rounded opacity-0 transition-all group-hover:opacity-100 text-zinc-500 hover:text-zinc-300 hover:bg-zinc-700",
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
                        div { class: "py-3 px-4 transition-colors group hover:bg-zinc-800/50",
                            div { class: "flex gap-3 items-center",
                                span { class: "flex-shrink-0 w-20 font-mono text-xs text-zinc-500",
                                    "[^newref]"
                                }
                                button { class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-500 hover:text-zinc-300",
                                    svg {
                                        class: "flex-shrink-0 w-4 h-4",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                        }
                                    }
                                    span { class: "italic", "Click to link note..." }
                                }
                            }
                        }
                        div { class: "py-3 px-4 transition-colors group hover:bg-zinc-800/50",
                            div { class: "flex gap-3 items-center",
                                span { class: "flex-shrink-0 w-20 font-mono text-xs text-zinc-500",
                                    "[^todo]"
                                }
                                button { class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-500 hover:text-zinc-300",
                                    svg {
                                        class: "flex-shrink-0 w-4 h-4",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                        }
                                    }
                                    span { class: "italic", "Click to link note..." }
                                }
                            }
                        }
                    }
                }
                div { class: "overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900/30",
                    div { class: "py-3 px-4 border-b border-zinc-800 bg-zinc-900/50",
                        h3 { class: "text-sm font-semibold text-zinc-300",
                            "\n                        Footnotes\n                    "
                        }
                    }
                    div { class: "py-8 px-4 text-center",
                        p { class: "text-sm italic text-zinc-500",
                            "\n                        No footnotes found. Use [^name] to add references.\n                    "
                        }
                    }
                }
                div { class: "overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900/30",
                    div { class: "flex justify-between items-center py-2 px-3 border-b border-zinc-800 bg-zinc-900/50",
                        h3 { class: "text-xs font-semibold text-zinc-300",
                            "\n                        Footnotes\n                    "
                        }
                        span { class: "text-xs text-zinc-500", "4 references" }
                    }
                    div { class: "divide-y divide-zinc-800",
                        div { class: "py-2 px-3 transition-colors hover:bg-zinc-800/50",
                            div { class: "flex gap-2 items-center",
                                span { class: "font-mono text-xs text-zinc-500", "[1]" }
                                button { class: "flex-1 text-xs text-left text-zinc-300 truncate",
                                    "\n                                Introduction to Systems\n                            "
                                }
                            }
                        }
                        div { class: "py-2 px-3 transition-colors hover:bg-zinc-800/50",
                            div { class: "flex gap-2 items-center",
                                span { class: "font-mono text-xs text-zinc-500", "[^2]" }
                                button { class: "flex-1 text-xs italic text-left text-zinc-500 truncate",
                                    "\n                                Click to link...\n                            "
                                }
                            }
                        }
                    }
                }
            }
        }
    }

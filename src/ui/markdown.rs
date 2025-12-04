use dioxus::prelude::*;
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};

#[derive(Clone)]
enum ParagraphElement {
    Text(String),
    Link { text: String, url: String },
}

#[component]
pub fn SimpleMarkdown(
    content: String,
    on_internal_link_click: EventHandler<String>,
) -> Element {
    let parser = Parser::new(&content);
    let mut elements = Vec::new();
    let mut current_text = String::new();
    let mut link_url: Option<String> = None;
    let mut link_text = String::new();
    let mut heading_level: Option<HeadingLevel> = None;
    let mut heading_text = String::new();
    let mut in_paragraph = false;
    let mut paragraph_elements: Vec<ParagraphElement> = Vec::new();
    let mut pending_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading_level {
                    let text = heading_text.clone();
                    let class_name = match level {
                        HeadingLevel::H1 => "text-3xl font-bold my-4",
                        HeadingLevel::H2 => "text-2xl font-bold my-3",
                        HeadingLevel::H3 => "text-xl font-bold my-3",
                        HeadingLevel::H4 => "text-lg font-bold my-2",
                        HeadingLevel::H5 => "text-base font-bold my-2",
                        HeadingLevel::H6 => "text-sm font-bold my-2",
                    };
                    elements.push(rsx! {
                        div { key: "{elements.len()}", class: "{class_name}", "{text}" }
                    });
                    heading_level = None;
                }
            }
            Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
                paragraph_elements.clear();
                pending_text.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if in_paragraph {
                    // Flush any pending text
                    if !pending_text.is_empty() {
                        paragraph_elements.push(ParagraphElement::Text(pending_text.clone()));
                        pending_text.clear();
                    }

                    // Render paragraph with mixed content
                    let para_elements = paragraph_elements.clone();
                    let key = elements.len();
                    elements.push(rsx! {
                        p { key: "{key}", class: "my-4",
                            {para_elements.iter().enumerate().map(|(i, elem)| {
                                match elem {
                                    ParagraphElement::Text(text) => rsx! {
                                        span { key: "{i}", "{text}" }
                                    },
                                    ParagraphElement::Link { text, url } => {
                                        let url = url.clone();
                                        rsx! {
                                            a {
                                                key: "{i}",
                                                class: "text-blue-600 underline cursor-pointer hover:text-blue-800",
                                                onclick: move |evt| {
                                                    evt.prevent_default();
                                                    on_internal_link_click.call(url.clone());
                                                },
                                                "{text}"
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                    });
                    in_paragraph = false;
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_url = Some(dest_url.to_string());
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                if let Some(url) = link_url.take() {
                    let text = link_text.clone();

                    if in_paragraph {
                        // Flush pending text before adding link
                        if !pending_text.is_empty() {
                            paragraph_elements.push(ParagraphElement::Text(pending_text.clone()));
                            pending_text.clear();
                        }
                        paragraph_elements.push(ParagraphElement::Link { text, url });
                    } else {
                        let url_clone = url.clone();
                        elements.push(rsx! {
                            a {
                                key: "{elements.len()}",
                                class: "text-blue-600 underline cursor-pointer hover:text-blue-800",
                                onclick: move |evt| {
                                    evt.prevent_default();
                                    on_internal_link_click.call(url_clone.clone());
                                },
                                "{text}"
                            }
                        });
                    }
                }
            }
            Event::Text(text) => {
                let text_str = text.to_string();
                if heading_level.is_some() {
                    heading_text.push_str(&text_str);
                } else if link_url.is_some() {
                    link_text.push_str(&text_str);
                } else if in_paragraph {
                    pending_text.push_str(&text_str);
                } else {
                    current_text.push_str(&text_str);
                }
            }
            Event::Code(code) => {
                let code_str = format!("`{}`", code);
                if in_paragraph {
                    pending_text.push_str(&code_str);
                } else {
                    current_text.push_str(&code_str);
                }
            }
            Event::Start(Tag::Strong) => {
                let marker = "**";
                if in_paragraph {
                    pending_text.push_str(marker);
                } else {
                    current_text.push_str(marker);
                }
            }
            Event::End(TagEnd::Strong) => {
                let marker = "**";
                if in_paragraph {
                    pending_text.push_str(marker);
                } else {
                    current_text.push_str(marker);
                }
            }
            Event::Start(Tag::Emphasis) => {
                let marker = "*";
                if in_paragraph {
                    pending_text.push_str(marker);
                } else {
                    current_text.push_str(marker);
                }
            }
            Event::End(TagEnd::Emphasis) => {
                let marker = "*";
                if in_paragraph {
                    pending_text.push_str(marker);
                } else {
                    current_text.push_str(marker);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_paragraph {
                    pending_text.push(' ');
                } else {
                    current_text.push(' ');
                }
            }
            _ => {
                // Ignore other events for now
            }
        }
    }

    // Flush any remaining text
    if !current_text.is_empty() {
        let text = current_text.clone();
        elements.push(rsx! {
            div { key: "{elements.len()}", "{text}" }
        });
    }

    rsx! {
        div { class: "p-4 markdown-content",
            {elements.into_iter()}
        }
    }
}

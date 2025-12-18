use dioxus::prelude::*;
use regex::Regex;

#[derive(Clone, PartialEq)]
enum TextSegment {
    Text(String),
    Footnote(usize),
}

#[component]
pub fn PlainTextViewer(content: String) -> Element {
    // Regex to match [1], [2], etc.
    let footnote_re = Regex::new(r"\[(\d+)\]").unwrap();

    // Split content into lines to preserve structure
    let lines: Vec<&str> = content.lines().collect();

    let mut elements = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        if line.is_empty() {
            // Preserve empty lines as spacing
            elements.push(rsx! {
                div { key: "{line_idx}", class: "h-4" }
            });
            continue;
        }

        // Parse line into text segments and footnote references
        let mut segments = Vec::new();
        let mut last_end = 0;

        for capture in footnote_re.captures_iter(line) {
            let match_obj = capture.get(0).unwrap();
            let start = match_obj.start();
            let end = match_obj.end();

            // Add text before the footnote
            if start > last_end {
                segments.push(TextSegment::Text(line[last_end..start].to_string()));
            }

            // Add the footnote reference
            if let Ok(num) = capture[1].parse::<usize>() {
                segments.push(TextSegment::Footnote(num));
            }

            last_end = end;
        }

        // Add remaining text after last footnote
        if last_end < line.len() {
            segments.push(TextSegment::Text(line[last_end..].to_string()));
        }

        // If no footnotes found, treat entire line as text
        if segments.is_empty() {
            segments.push(TextSegment::Text(line.to_string()));
        }

        // Render the line with segments
        elements.push(rsx! {
            div { key: "{line_idx}", class: "my-1",
                {segments.iter().enumerate().map(|(seg_idx, segment)| {
                    match segment {
                        TextSegment::Text(text) => rsx! {
                            span { key: "{seg_idx}", "{text}" }
                        },
                        TextSegment::Footnote(num) => rsx! {
                            span {
                                key: "{seg_idx}",
                                class: "text-app-primary-light font-medium",
                                "[{num}]"
                            }
                        }
                    }
                })}
            }
        });
    }

    rsx! {
        div { class: "p-4 text-app-text whitespace-pre-wrap font-mono text-sm",
            {elements.into_iter()}
        }
    }
}

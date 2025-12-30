use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

use super::lamport_timestamp::LamportTimestamp;

#[derive(Debug, Clone)]
pub struct Note {
    pub frontmatter: Frontmatter,
    pub content: String,
    pub footnotes: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub uuid: Uuid,
    pub modified: LamportTimestamp,
    #[serde(default)]
    pub share_with: Vec<String>,
    #[serde(flatten)]
    extra: serde_yaml::Value,
}

impl Note {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read note: {}", path.as_ref().display()))?;
        Self::from_string(&content)
    }

    pub fn from_string(content: &str) -> Result<Self> {
        if !content.starts_with("---\n") {
            anyhow::bail!("Missing YAML frontmatter start: (---)")
        }
        let remaining = &content[4..];
        let end_pos = match remaining.find("\n---\n") {
            Some(pos) => pos,
            None => anyhow::bail!("Missing YAML front matter end: ---"),
        };
        let yaml_str = &remaining[..end_pos];

        let frontmatter: Frontmatter = serde_yaml::from_str(yaml_str)?;
        let content_start = end_pos + 5; // Skip "\n---\n"
        let full_content = remaining[content_start..].trim_start();
        let (body, footnotes) = Self::parse_body_and_footnotes(full_content);

        Ok(Note {
            frontmatter,
            content: body,
            footnotes,
        })
    }

    /// To start, we will support a specific markdown compatible but restricted
    /// format. One link per line, footnotes must be at the end of the file
    /// (newline)
    /// [^(footnotename)]: footnote body
    /// [^(footnotename2)]: footnote body2

    fn parse_body_and_footnotes(content: &str) -> (String, IndexMap<String, String>) {
        let lines: Vec<&str> = content.lines().collect();
        let mut footnotes = IndexMap::new();

        let mut footnote_start_idx = None;
        for idx in (0..lines.len()).rev() {
            let trimmed = lines[idx].trim();

            if trimmed.starts_with("[^") && trimmed.contains("]:") {
                footnote_start_idx = Some(idx);
            } else if !trimmed.is_empty() && footnote_start_idx.is_some() {
                break;
            }
        }

        let (body_lines, footnote_lines) = match footnote_start_idx {
            Some(start_idx) => (&lines[..start_idx], &lines[start_idx..]),
            None => (&lines[..], &[] as &[&str]),
        };

        for line in footnote_lines {
            if let Some((id, text)) = Self::parse_footnote_line(line) {
                footnotes.insert(id, text);
            }
        }

        let body = body_lines.join("\n");
        (body, footnotes)
    }

    fn parse_footnote_line(line: &str) -> Option<(String, String)> {
        let rest = &line[2..]; // Skip "[^"
        let close_bracket = rest.find("]:")?;

        let id = rest[..close_bracket].to_string();
        let text = rest[close_bracket + 2..].trim().to_string();

        Some((id, text))
    }

    pub fn to_string(&self) -> Result<String> {
        let yaml =
            serde_yaml::to_string(&self.frontmatter).context("Failed to serialize frontmatter")?;

        let mut result = format!("---\n{}---\n\n{}", yaml, self.content);

        // Add footnotes at the bottom if there are any
        if !self.footnotes.is_empty() {
            result.push_str("\n\n");
            for (id, text) in &self.footnotes {
                result.push_str(&format!("[^{}]: {}\n", id, text));
            }
        }

        Ok(result)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let serialized = self.to_string()?;
        fs::write(path.as_ref(), serialized)
            .with_context(|| format!("Failed to write note: {}", path.as_ref().display()))?;
        Ok(())
    }

    pub fn create(path: &Path, content: &str) -> Result<Self> {
        let (body, footnotes) = Self::parse_body_and_footnotes(&content);

        let frontmatter = Frontmatter {
            uuid: Uuid::new_v4(),
            modified: LamportTimestamp::now(),
            share_with: Vec::new(),
            extra: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        };

        let note = Note {
            frontmatter,
            content: body,
            footnotes,
        };

        note.save(path)?;
        Ok(note)
    }

    pub fn update(&mut self, path: &Path, content: &str) -> Result<()> {
        let (body, footnotes) = Self::parse_body_and_footnotes(&content);
        self.content = body;
        self.footnotes = footnotes;
        self.frontmatter.modified = LamportTimestamp::new(Some(self.frontmatter.modified));
        self.save(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_with_frontmatter() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with:
  - alice
  - bob
---

# My Note

This is the content.
"#;

        let note = Note::from_string(content).unwrap();
        assert_eq!(
            note.frontmatter.uuid.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(note.frontmatter.share_with, vec!["alice", "bob"]);
        assert_eq!(note.content, "# My Note\n\nThis is the content.");
        assert_eq!(note.footnotes.len(), 0);
    }

    #[test]
    fn test_parse_note_without_frontmatter_fails() {
        let content = "# My Note\n\nNo frontmatter here.";
        assert!(Note::from_string(content).is_err());
    }

    #[test]
    fn test_parse_note_with_footnotes() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
---

This is some text with [^1] and [^second] references.

[^1]: First footnote text
[^second]: Second footnote text
"#;

        let note = Note::from_string(content).unwrap();
        assert_eq!(note.footnotes.len(), 2);
        assert_eq!(
            note.footnotes.get("1"),
            Some(&"First footnote text".to_string())
        );
        assert_eq!(
            note.footnotes.get("second"),
            Some(&"Second footnote text".to_string())
        );
        assert_eq!(
            note.content,
            "This is some text with [^1] and [^second] references.\n"
        );
    }

    #[test]
    fn test_footnote_order_preserved() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
---

Text with refs.

[^third]: Third
[^first]: First
[^second]: Second
"#;

        let note = Note::from_string(content).unwrap();
        let keys: Vec<&String> = note.footnotes.keys().collect();
        assert_eq!(keys, vec!["third", "first", "second"]);
    }

    #[test]
    fn test_serialize_note() {
        let mut frontmatter_map = serde_yaml::Mapping::new();
        frontmatter_map.insert(
            serde_yaml::Value::String("uuid".to_string()),
            serde_yaml::Value::String("550e8400-e29b-41d4-a716-446655440000".to_string()),
        );
        frontmatter_map.insert(
            serde_yaml::Value::String("modified".to_string()),
            serde_yaml::Value::Number(1705316400.into()),
        );
        frontmatter_map.insert(
            serde_yaml::Value::String("share_with".to_string()),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("alice".to_string())]),
        );

        let frontmatter = Frontmatter {
            uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            modified: LamportTimestamp(1705316400),
            share_with: vec!["alice".to_string()],
            extra: serde_yaml::Value::Mapping(frontmatter_map),
        };

        let note = Note {
            frontmatter,
            content: "# Content".to_string(),
            footnotes: IndexMap::new(),
        };

        let serialized = note.to_string().unwrap();
        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("uuid:"));
        assert!(serialized.contains("# Content"));
    }

    #[test]
    fn test_round_trip_preserves_unknown_yaml() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
custom_field: some_value
tags:
  - important
  - work
---

Content here.
"#;

        let note = Note::from_string(content).unwrap();
        let serialized = note.to_string().unwrap();

        assert!(serialized.contains("custom_field"));
        assert!(serialized.contains("tags"));
    }

    #[test]
    fn test_footnote_round_trip() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
---

Content with [^1] reference.

[^1]: Footnote text here
"#;

        let note = Note::from_string(content).unwrap();
        let serialized = note.to_string().unwrap();
        let reparsed = Note::from_string(&serialized).unwrap();

        assert_eq!(note.footnotes, reparsed.footnotes);
    }
}

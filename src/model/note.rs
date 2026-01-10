use crate::util::lamport_timestamp::LamportTimestamp;
use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Note {
    pub frontmatter: Frontmatter,
    pub content: String,
    pub footnotes: IndexMap<String, String>,
    loaded_from: Option<PathBuf>,
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
    pub fn from_path(path: impl AsRef<Path>, coerce_to_footnote: bool) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read note: {}", path.as_ref().display()))?;
        let mut note = Self::from_string(&content, coerce_to_footnote)?;
        note.loaded_from = Some(path.as_ref().to_path_buf());
        Ok(note)
    }

    pub fn from_string(content: &str, coerce_to_footnote: bool) -> Result<Self> {
        let (frontmatter, content_start) = match Self::parse_frontmatter(content) {
            Ok(r) => r,
            Err(e) => {
                if coerce_to_footnote {
                    tracing::warn!("Failed to parse frontmatter, creating new: {}", e);
                    (Self::create_frontmatter(), 0)
                } else {
                    anyhow::bail!("failed to parse frontmatter");
                }
            }
        };

        let full_content = if content_start > 0 {
            content[content_start..].trim_start()
        } else {
            content.trim_start()
        };

        let (body, footnotes) = Self::parse_body_and_footnotes(full_content);

        Ok(Note {
            frontmatter,
            content: body,
            footnotes,
            loaded_from: None,
        })
    }

    fn parse_frontmatter(content: &str) -> Result<(Frontmatter, usize)> {
        if !content.starts_with("---\n") {
            anyhow::bail!("Missing YAML frontmatter start");
        }
        let remaining = &content[4..];
        let end_pos = remaining
            .find("\n---\n")
            .ok_or_else(|| anyhow::anyhow!("Missing YAML frontmatter end"))?;

        let yaml_str = &remaining[..end_pos];
        let frontmatter: Frontmatter = serde_yaml::from_str(yaml_str)?;
        let content_start = 4 + end_pos + 5;

        Ok((frontmatter, content_start))
    }

    /// To start, we will support a specific markdown compatible but restricted
    /// format. One link per line, footnotes must be at the end of the file
    /// (newline)
    /// [\d]: footnote body
    fn parse_body_and_footnotes(content: &str) -> (String, IndexMap<String, String>) {
        let lines: Vec<&str> = content.lines().collect();
        let mut footnotes = IndexMap::new();

        let mut footnote_start_idx = None;
        for idx in (0..lines.len()).rev() {
            let trimmed = lines[idx].trim();

            if trimmed.starts_with("[") && trimmed.contains("]:") {
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
        let rest = &line[1..];
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
                result.push_str(&format!("[{}]: {}\n", id, text));
            }
        }

        Ok(result)
    }

    pub fn to_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        self.frontmatter.modified = LamportTimestamp::new(Some(self.frontmatter.modified));

        if let Some(old_path) = &self.loaded_from {
            if old_path != path && old_path.exists() {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let temp_path = path.with_extension("tmp");
                let serialized = self.to_string()?;
                fs::write(&temp_path, serialized)?;
                fs::rename(&temp_path, path)?;
                fs::remove_file(old_path)?;
                self.loaded_from = Some(path.to_path_buf());
                return Ok(());
            }
        }

        let serialized = self.to_string()?;
        fs::write(path, serialized)?;
        self.loaded_from = Some(path.to_path_buf());
        Ok(())
    }

    pub fn create(path: &Path, content: &str) -> Result<Self> {
        let (body, footnotes) = Self::parse_body_and_footnotes(&content);

        let frontmatter = Note::create_frontmatter();
        let mut note = Note {
            frontmatter,
            content: body,
            footnotes,
            loaded_from: Some(path.to_path_buf()),
        };

        note.to_file(path)?;
        Ok(note)
    }

    fn create_frontmatter() -> Frontmatter {
        Frontmatter {
            uuid: Uuid::new_v4(),
            modified: LamportTimestamp::now(),
            share_with: Vec::new(),
            extra: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        }
    }

    pub fn update(&mut self, path: &Path, content: &str) -> Result<()> {
        let (body, footnotes) = Self::parse_body_and_footnotes(&content);
        self.content = body;
        self.footnotes = footnotes;
        self.frontmatter.modified = LamportTimestamp::new(Some(self.frontmatter.modified));
        self.to_file(path)?;
        Ok(())
    }

    pub fn update_all(
        &mut self,
        path: &Path,
        body: &str,
        footnotes: IndexMap<String, String>,
    ) -> Result<()> {
        self.content = body.to_string();
        self.footnotes = footnotes;
        self.frontmatter.modified = LamportTimestamp::new(Some(self.frontmatter.modified));
        self.to_file(path)?;
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

        let note = Note::from_string(content, false).unwrap();
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
        assert!(Note::from_string(content, false).is_err());
    }

    #[test]
    fn test_parse_note_without_frontmatter_can_request_frontmatter() {
        let content = "# My Note\n\nNo frontmatter here.";
        assert!(Note::from_string(content, true).is_ok());
    }

    #[test]
    fn test_parse_note_with_footnotes() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
---

This is some text with one [1] and two [2] references.

[1]: First footnote text
[2]: Second footnote text
"#;

        let note = Note::from_string(content, false).unwrap();
        assert_eq!(note.footnotes.len(), 2);
        assert_eq!(
            note.footnotes.get("1"),
            Some(&"First footnote text".to_string())
        );
        assert_eq!(
            note.footnotes.get("2"),
            Some(&"Second footnote text".to_string())
        );
        assert_eq!(
            note.content,
            "This is some text with one [1] and two [2] references.\n"
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

[3]: Third
[1]: First
[2]: Second
"#;

        let note = Note::from_string(content, false).unwrap();
        let keys: Vec<&String> = note.footnotes.keys().collect();
        assert_eq!(keys, vec!["3", "1", "2"]);
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
            loaded_from: None,
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

        let note = Note::from_string(content, false).unwrap();
        let serialized = note.to_string().unwrap();

        assert!(serialized.contains("custom_field"));
        assert!(serialized.contains("tags"));
    }

    #[test]
    fn test_imported_document_preserves_unknown_yaml() {
        let content = r#"---
custom_field: some_value
tags:
  - important
  - work
---

Content here.
"#;

        let note = Note::from_string(content, true).unwrap();
        let serialized = note.to_string().unwrap();

        assert!(serialized.contains("id"));
        assert!(serialized.contains("share_with"));
        assert!(serialized.contains("modified"));
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

Content with [1] reference.


[1]: Footnote text here
"#;

        let note = Note::from_string(content, false).unwrap();
        let serialized = note.to_string().unwrap();
        let reparsed = Note::from_string(&serialized, false).unwrap();

        assert_eq!(note.footnotes, reparsed.footnotes);
    }
}

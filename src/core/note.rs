use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Vector time for tracking modifications with causality
/// Uses max(file_modified_time + 1, unix_time) for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct VectorTime(pub i64);

impl VectorTime {
    /// Create a new vector time from a previous time
    /// Returns max(previous_time + 1, current_unix_time)
    pub fn new(previous: Option<VectorTime>) -> Self {
        let current_unix = Utc::now().timestamp();
        match previous {
            Some(VectorTime(prev)) => VectorTime(std::cmp::max(prev + 1, current_unix)),
            None => VectorTime(current_unix),
        }
    }

    /// Get the unix timestamp value
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl Default for VectorTime {
    fn default() -> Self {
        VectorTime::new(None)
    }
}

/// A footnote reference linking to another note by UUID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Footnote {
    pub number: usize,
    pub title: String,
    pub uuid: Uuid,
}

/// Frontmatter metadata for a note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteFrontmatter {
    #[serde(default = "generate_uuid")]
    pub uuid: Uuid,

    #[serde(
        default = "default_vector_time",
        deserialize_with = "deserialize_vector_time_with_fallback"
    )]
    pub modified: VectorTime,

    #[serde(default)]
    pub share_with: Vec<String>,

    #[serde(default)]
    pub footnotes: Vec<Footnote>,
}

fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

fn default_vector_time() -> VectorTime {
    VectorTime::default()
}

/// Custom deserializer for VectorTime that falls back to current timestamp on error
fn deserialize_vector_time_with_fallback<'de, D>(deserializer: D) -> Result<VectorTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    // Try to deserialize as i64
    match i64::deserialize(deserializer) {
        Ok(timestamp) => Ok(VectorTime(timestamp)),
        Err(_) => {
            // Fall back to current timestamp
            Ok(VectorTime::default())
        }
    }
}

impl Default for NoteFrontmatter {
    fn default() -> Self {
        Self {
            uuid: generate_uuid(),
            modified: default_vector_time(),
            share_with: Vec::new(),
            footnotes: Vec::new(),
        }
    }
}

/// A parsed note with frontmatter and content
#[derive(Debug)]
pub struct Note {
    pub frontmatter: NoteFrontmatter,
    pub content: String,
}

/// Parse a markdown file and extract frontmatter
///
/// Expects frontmatter in YAML format between `---` delimiters:
/// ```markdown
/// ---
/// uuid: 550e8400-e29b-41d4-a716-446655440000
/// modified: 2024-01-15T10:30:00Z
/// share_with:
///   - alice
///   - bob
/// ---
/// # Document content here
/// ```
///
/// If no frontmatter is found, returns default frontmatter with a new UUID
pub fn parse_note(path: &Path) -> Result<Note> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read note: {}", path.display()))?;

    parse_note_from_string(&content)
}

/// Parse note content from a string
pub fn parse_note_from_string(content: &str) -> Result<Note> {
    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        // No frontmatter - return default
        return Ok(Note {
            frontmatter: NoteFrontmatter::default(),
            content: content.to_string(),
        });
    }

    // Find the end of frontmatter
    let rest = if content.starts_with("---\r\n") {
        &content[5..]
    } else {
        &content[4..]
    };

    if let Some(end_pos) = rest.find("\n---\n").or_else(|| rest.find("\r\n---\r\n")) {
        // Extract frontmatter YAML
        let yaml_str = &rest[..end_pos];
        let frontmatter: NoteFrontmatter =
            serde_yaml::from_str(yaml_str).context("Failed to parse frontmatter YAML")?;

        // Extract content after frontmatter
        let content_start = if rest[end_pos..].starts_with("\r\n") {
            end_pos + 7 // Skip "\r\n---\r\n"
        } else {
            end_pos + 5 // Skip "\n---\n"
        };
        let content = rest[content_start..].to_string();

        Ok(Note {
            frontmatter,
            content,
        })
    } else {
        // Malformed frontmatter - treat entire file as content
        Ok(Note {
            frontmatter: NoteFrontmatter::default(),
            content: content.to_string(),
        })
    }
}

/// Serialize a note back to markdown with frontmatter
pub fn serialize_note(note: &Note) -> Result<String> {
    let yaml =
        serde_yaml::to_string(&note.frontmatter).context("Failed to serialize frontmatter")?;

    Ok(format!("---\n{}---\n{}", yaml, note.content))
}

/// Update the frontmatter of a file
pub fn update_frontmatter(path: &Path, frontmatter: NoteFrontmatter) -> Result<()> {
    let mut note = parse_note(path)?;
    note.frontmatter = frontmatter;

    let serialized = serialize_note(&note)?;
    fs::write(path, serialized)
        .with_context(|| format!("Failed to write note: {}", path.display()))?;

    Ok(())
}

/// Get just the frontmatter from a file without parsing the full content
pub fn get_frontmatter(path: &Path) -> Result<NoteFrontmatter> {
    let note = parse_note(path)?;
    Ok(note.frontmatter)
}

/// Find a note file by UUID in a directory
/// Scans all .md files in the directory and returns the path of the file with matching UUID
pub fn find_note_by_uuid(dir: &Path, target_uuid: &Uuid) -> Result<Option<PathBuf>> {
    if !dir.is_dir() {
        return Ok(None);
    }

    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Only check .md files
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        // Skip if not a file
        if !path.is_file() {
            continue;
        }

        // Parse the note and check UUID
        if let Ok(note) = parse_note(&path) {
            if note.frontmatter.uuid == *target_uuid {
                return Ok(Some(path));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_with_frontmatter() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 2024-01-15T10:30:00Z
share_with:
  - alice
  - bob
---
# My Note

This is the content.
"#;

        let note = parse_note_from_string(content).unwrap();
        assert_eq!(
            note.frontmatter.uuid.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(note.frontmatter.share_with, vec!["alice", "bob"]);
        assert_eq!(note.content, "# My Note\n\nThis is the content.\n");
    }

    #[test]
    fn test_parse_note_without_frontmatter() {
        let content = "# My Note\n\nNo frontmatter here.";

        let note = parse_note_from_string(content).unwrap();
        assert_eq!(note.content, content);
        assert_eq!(note.frontmatter.share_with.len(), 0);
    }

    #[test]
    fn test_serialize_note() {
        let frontmatter = NoteFrontmatter {
            uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            modified: VectorTime(1705316400),
            share_with: vec!["alice".to_string()],
            footnotes: vec![],
        };

        let note = Note {
            frontmatter,
            content: "# Content".to_string(),
        };

        let serialized = serialize_note(&note).unwrap();
        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("uuid:"));
        assert!(serialized.contains("# Content"));
    }

    #[test]
    fn test_parse_note_with_invalid_modified_field() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: "invalid-timestamp"
share_with: []
---
# Test Note

This note has an invalid modified timestamp.
"#;

        let note = parse_note_from_string(content).unwrap();
        // Should have valid frontmatter with defaulted timestamp
        assert_eq!(
            note.frontmatter.uuid.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        // modified should be a valid VectorTime (current timestamp)
        assert!(note.frontmatter.modified.as_i64() > 0);
        assert_eq!(
            note.content,
            "# Test Note\n\nThis note has an invalid modified timestamp.\n"
        );
    }

    #[test]
    fn test_parse_note_with_datetime_string_modified() {
        // Test that old DateTime RFC3339 strings gracefully fall back to current timestamp
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 2024-01-15T10:30:00Z
share_with: []
---
# Old Note

This note has a DateTime string in the modified field.
"#;

        let note = parse_note_from_string(content).unwrap();
        // Should parse without error
        assert_eq!(
            note.frontmatter.uuid.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        // modified should be a valid VectorTime (falls back to current timestamp)
        assert!(note.frontmatter.modified.as_i64() > 0);
    }

    #[test]
    fn test_parse_note_with_missing_modified_field() {
        // Test that missing modified field uses default
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
share_with: []
---
# Note Without Modified

This note has no modified field.
"#;

        let note = parse_note_from_string(content).unwrap();
        // Should parse without error
        assert_eq!(
            note.frontmatter.uuid.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        // modified should be a valid VectorTime (default)
        assert!(note.frontmatter.modified.as_i64() > 0);
    }

    #[test]
    fn test_parse_note_with_footnotes() {
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
footnotes:
  - number: 1
    title: "First Note"
    uuid: 11111111-1111-1111-1111-111111111111
  - number: 2
    title: "Second Note"
    uuid: 22222222-2222-2222-2222-222222222222
---
This is some text with [1] and [2] references.
"#;

        let note = parse_note_from_string(content).unwrap();
        assert_eq!(note.frontmatter.footnotes.len(), 2);
        assert_eq!(note.frontmatter.footnotes[0].number, 1);
        assert_eq!(note.frontmatter.footnotes[0].title, "First Note");
        assert_eq!(
            note.frontmatter.footnotes[0].uuid.to_string(),
            "11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(note.frontmatter.footnotes[1].number, 2);
        assert_eq!(note.frontmatter.footnotes[1].title, "Second Note");
    }

    #[test]
    fn test_parse_note_without_footnotes_field() {
        // Test backward compatibility - notes without footnotes field should parse
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
---
# Note Without Footnotes

This note has no footnotes field.
"#;

        let note = parse_note_from_string(content).unwrap();
        // Should parse without error, footnotes should be empty
        assert_eq!(note.frontmatter.footnotes.len(), 0);
    }

    #[test]
    fn test_serialize_note_with_footnotes() {
        let footnote1 = Footnote {
            number: 1,
            title: "Test Note".to_string(),
            uuid: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        };

        let frontmatter = NoteFrontmatter {
            uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            modified: VectorTime(1705316400),
            share_with: vec![],
            footnotes: vec![footnote1],
        };

        let note = Note {
            frontmatter,
            content: "Text with [1] reference.".to_string(),
        };

        let serialized = serialize_note(&note).unwrap();
        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("footnotes:"));
        assert!(serialized.contains("number: 1"));
        assert!(serialized.contains("title: Test Note"));
        assert!(serialized.contains("11111111-1111-1111-1111-111111111111"));
        assert!(serialized.contains("Text with [1] reference."));
    }

    #[test]
    fn test_footnote_round_trip() {
        // Test that footnotes survive serialization and deserialization
        let footnote = Footnote {
            number: 1,
            title: "Round Trip".to_string(),
            uuid: Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap(),
        };

        let frontmatter = NoteFrontmatter {
            uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            modified: VectorTime(1705316400),
            share_with: vec![],
            footnotes: vec![footnote.clone()],
        };

        let note = Note {
            frontmatter,
            content: "Content".to_string(),
        };

        let serialized = serialize_note(&note).unwrap();
        let parsed = parse_note_from_string(&serialized).unwrap();

        assert_eq!(parsed.frontmatter.footnotes.len(), 1);
        assert_eq!(parsed.frontmatter.footnotes[0], footnote);
    }
}

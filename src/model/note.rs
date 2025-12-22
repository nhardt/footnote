use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Clone, PartialEq)]
pub struct FootnoteFile {
    pub path: PathBuf,
    pub title: String,
    pub uuid: String,
    pub share_with: Vec<String>,
    /// all frontmatter. uuid and share_with are duplicated here, but will be
    /// overwritten with the values from this struct on write.
    pub frontmatter: String,
    pub body: String,
    pub footnotes: Vec<Footnote>,
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
        }
    }
}

/// A parsed note with frontmatter, content, and footnotes
#[derive(Debug)]
pub struct Note {
    pub frontmatter: NoteFrontmatter,
    pub content: String,
    pub footnotes: Vec<Footnote>,
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
        // No frontmatter - parse footnotes and return
        let (body, footnotes) = parse_footnotes(content);
        return Ok(Note {
            frontmatter: NoteFrontmatter::default(),
            content: body,
            footnotes,
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
        let full_content = &rest[content_start..];

        // Parse footnotes from content
        let (body, footnotes) = parse_footnotes(full_content);

        Ok(Note {
            frontmatter,
            content: body,
            footnotes,
        })
    } else {
        // Malformed frontmatter - treat entire file as content
        let (body, footnotes) = parse_footnotes(content);
        Ok(Note {
            frontmatter: NoteFrontmatter::default(),
            content: body,
            footnotes,
        })
    }
}

/// Parse footnotes from content in the format: [1]: uuid:550e8400-... "Title"
/// Returns (content_without_footnotes, footnotes)
fn parse_footnotes(content: &str) -> (String, Vec<Footnote>) {
    let mut footnotes = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Find where footnotes section starts (first line matching footnote pattern)
    let mut footnote_start_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim().starts_with('[') && line.contains("]: uuid:") {
            footnote_start_idx = Some(idx);
            break;
        }
    }

    let (body_lines, footnote_lines) = if let Some(start_idx) = footnote_start_idx {
        // Trim empty lines before footnotes section
        let mut body_end = start_idx;
        while body_end > 0 && lines[body_end - 1].trim().is_empty() {
            body_end -= 1;
        }
        (&lines[..body_end], &lines[start_idx..])
    } else {
        // No footnotes found
        (&lines[..], &[] as &[&str])
    };

    // Parse footnote lines: [1]: uuid:550e8400-e29b-41d4-a716-446655440000 "Title"
    for line in footnote_lines {
        if let Some(parsed) = parse_footnote_line(line) {
            footnotes.push(parsed);
        }
    }

    let body = body_lines.join("\n");
    (body, footnotes)
}

/// Parse a single footnote line: [1]: uuid:550e8400-... "Title"
fn parse_footnote_line(line: &str) -> Option<Footnote> {
    let line = line.trim();

    // Match pattern: [number]: uuid:UUID "Title"
    if !line.starts_with('[') {
        return None;
    }

    let rest = &line[1..];
    let close_bracket = rest.find(']')?;
    let number_str = &rest[..close_bracket];
    let number: usize = number_str.parse().ok()?;

    let after_bracket = &rest[close_bracket + 1..].trim();
    if !after_bracket.starts_with(": uuid:") {
        return None;
    }

    let uuid_and_title = &after_bracket[7..]; // Skip ": uuid:"

    // Find the space before the title (title is in quotes)
    let quote_start = uuid_and_title.find('"')?;
    let uuid_str = uuid_and_title[..quote_start].trim();
    let uuid = Uuid::parse_str(uuid_str).ok()?;

    // Extract title from quotes
    let after_quote = &uuid_and_title[quote_start + 1..];
    let quote_end = after_quote.find('"')?;
    let title = after_quote[..quote_end].to_string();

    Some(Footnote {
        number,
        title,
        uuid,
    })
}

/// Serialize a note back to markdown with frontmatter and footnotes
pub fn serialize_note(note: &Note) -> Result<String> {
    let yaml =
        serde_yaml::to_string(&note.frontmatter).context("Failed to serialize frontmatter")?;

    let mut result = format!("---\n{}---\n{}", yaml, note.content);

    // Add footnotes at the bottom if there are any
    if !note.footnotes.is_empty() {
        result.push_str("\n\n");
        for footnote in &note.footnotes {
            result.push_str(&format!(
                "[{}]: uuid:{} \"{}\"\n",
                footnote.number, footnote.uuid, footnote.title
            ));
        }
    }

    Ok(result)
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

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with('.') {
                continue;
            }
        }

        if let Ok(frontmatter) = note::get_frontmatter(path) {
            if frontmatter.uuid == *target_uuid {
                return Ok(Some(path.to_path_buf()));
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
        assert_eq!(note.content, "# My Note\n\nThis is the content.");
        assert_eq!(note.footnotes.len(), 0);
    }

    #[test]
    fn test_parse_note_without_frontmatter() {
        let content = "# My Note\n\nNo frontmatter here.";

        let note = parse_note_from_string(content).unwrap();
        assert_eq!(note.content, content);
        assert_eq!(note.frontmatter.share_with.len(), 0);
        assert_eq!(note.footnotes.len(), 0);
    }

    #[test]
    fn test_serialize_note() {
        let frontmatter = NoteFrontmatter {
            uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            modified: VectorTime(1705316400),
            share_with: vec!["alice".to_string()],
        };

        let note = Note {
            frontmatter,
            content: "# Content".to_string(),
            footnotes: vec![],
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
            "# Test Note\n\nThis note has an invalid modified timestamp."
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
---
This is some text with [1] and [2] references.

[1]: uuid:11111111-1111-1111-1111-111111111111 "First Note"
[2]: uuid:22222222-2222-2222-2222-222222222222 "Second Note"
"#;

        let note = parse_note_from_string(content).unwrap();
        assert_eq!(note.footnotes.len(), 2);
        assert_eq!(note.footnotes[0].number, 1);
        assert_eq!(note.footnotes[0].title, "First Note");
        assert_eq!(
            note.footnotes[0].uuid.to_string(),
            "11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(note.footnotes[1].number, 2);
        assert_eq!(note.footnotes[1].title, "Second Note");
        assert_eq!(
            note.content,
            "This is some text with [1] and [2] references."
        );
    }

    #[test]
    fn test_parse_note_without_footnotes_field() {
        // Test backward compatibility - notes without footnotes should parse
        let content = r#"---
uuid: 550e8400-e29b-41d4-a716-446655440000
modified: 1705316400
share_with: []
---
# Note Without Footnotes

This note has no footnotes.
"#;

        let note = parse_note_from_string(content).unwrap();
        // Should parse without error, footnotes should be empty
        assert_eq!(note.footnotes.len(), 0);
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
        };

        let note = Note {
            frontmatter,
            content: "Text with [1] reference.".to_string(),
            footnotes: vec![footnote1],
        };

        let serialized = serialize_note(&note).unwrap();
        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("uuid:"));
        assert!(serialized.contains("Text with [1] reference."));
        assert!(serialized.contains("[1]: uuid:11111111-1111-1111-1111-111111111111 \"Test Note\""));
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
        };

        let note = Note {
            frontmatter,
            content: "Content".to_string(),
            footnotes: vec![footnote.clone()],
        };

        let serialized = serialize_note(&note).unwrap();
        let parsed = parse_note_from_string(&serialized).unwrap();

        assert_eq!(parsed.footnotes.len(), 1);
        assert_eq!(parsed.footnotes[0], footnote);
    }
}

//! Lore Service
//! 
//! Handles Markdown parsing, YAML frontmatter extraction, and lore asset management.

use crate::models::LoreEntryResponse;
use crate::utils::error::{AppError, Result};
use std::path::Path;

/// Parse a Markdown file with YAML frontmatter into a LoreEntry
pub fn parse_lore_file(path: &Path) -> Result<Option<LoreEntryResponse>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Io(format!("Failed to read {}: {}", path.display(), e)))?;

    // Try to extract frontmatter — first YAML (---), then TOML-style
    let (metadata, body) = if content.starts_with("---") {
        parse_yaml_frontmatter(&content)
    } else {
        parse_toml_frontmatter(&content)
    };

    // Extract fields from metadata
    let title = metadata
        .as_ref()
        .and_then(|m| m.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled"),
        )
        .to_string();

    let lore_type = metadata
        .as_ref()
        .and_then(|m| m.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("story")
        .to_string();

    // Extract card names from YAML related_cards field
    let mut related_cards: Vec<String> = metadata
        .as_ref()
        .and_then(|m| m.get("related_cards"))
        .and_then(|v| {
            if let Some(arr) = v.as_array() {
                Some(
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                )
            } else if let Some(s) = v.as_str() {
                Some(s.split(',').map(|s| s.trim().to_string()).collect())
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Also extract card names from body: [[CardName]] and @CardName patterns
    let body_card_names = extract_card_names_from_body(body);
    for name in body_card_names {
        if !related_cards.contains(&name) {
            related_cards.push(name);
        }
    }

    Ok(Some(LoreEntryResponse {
        id: 0,
        title,
        lore_type,
        content: body.to_string(),
        metadata: None, // metadata already embedded in content, skip duplicate
        related_cards,
    }))
}

/// Parse YAML frontmatter between --- delimiters
fn parse_yaml_frontmatter(content: &str) -> (Option<serde_json::Value>, &str) {
    if let Some(end) = content[3..].find("\n---") {
        let yaml_str = &content[3..3 + end];
        let body = &content[3 + end + 5..]; // skip ---, yaml, \n---, and optional \n
        let metadata: serde_json::Value = match serde_yaml::from_str(yaml_str) {
            Ok(v) => v,
            Err(_) => {
                // Manual fallback: parse simple YAML key: value pairs
                let mut map = serde_json::Map::new();
                for line in yaml_str.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        let k = key.trim().to_string();
                        let v = value.trim().trim_matches('"').to_string();
                        map.insert(k, serde_json::Value::String(v));
                    }
                }
                serde_json::Value::Object(map)
            }
        };
        (Some(metadata), body.trim())
    } else {
        (None, content)
    }
}

/// Parse TOML-style frontmatter (key: value pairs before first blank line, no --- delimiters)
fn parse_toml_frontmatter(content: &str) -> (Option<serde_json::Value>, &str) {
    let mut meta_map = serde_json::Map::new();
    let mut meta_byte_end: usize = 0;
    let mut all_meta_valid = true;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Blank line separates metadata from body
            meta_byte_end += line.len() + 1; // +1 for newline char
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let k = key.trim();
            // Only accept simple alphanumeric keys (not markdown headings)
            if !k.is_empty() && k.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let v = value.trim().trim_matches('"').to_string();
                meta_map.insert(k.to_string(), serde_json::Value::String(v));
                meta_byte_end += line.len() + 1;
                continue;
            }
        }
        // If a line doesn't match the key:value pattern, it's not TOML frontmatter
        all_meta_valid = false;
        break;
    }

    if all_meta_valid && !meta_map.is_empty() {
        let body = content[meta_byte_end..].trim();
        (Some(serde_json::Value::Object(meta_map)), body)
    } else {
        (None, content)
    }
}

/// Extract card names from body text matching [[CardName]] and @CardName patterns
fn extract_card_names_from_body(body: &str) -> Vec<String> {
    let mut names = Vec::new();

    // Pattern: [[CardName]]
    let mut search_start = 0;
    while let Some(start) = body[search_start..].find("[[") {
        let abs_start = search_start + start + 2;
        if let Some(end_offset) = body[abs_start..].find("]]") {
            let name = body[abs_start..abs_start + end_offset].trim();
            if !name.is_empty() && !names.contains(&name.to_string()) {
                names.push(name.to_string());
            }
            search_start = abs_start + end_offset + 2;
        } else {
            break;
        }
    }

    // Pattern: @CardName (word starting with @ followed by an uppercase letter;
    //                continues adding words that start with uppercase letters)
    search_start = 0;
    while let Some(at_pos) = body[search_start..].find('@') {
        let abs_pos = search_start + at_pos;
        let after_at = &body[abs_pos + 1..];
        if let Some(next_char) = after_at.chars().next() {
            if next_char.is_ascii_uppercase() {
                // Collect consecutive uppercase-starting words
                let mut name = String::new();
                for word in after_at.split_whitespace() {
                    if let Some(c) = word.chars().next() {
                        if c.is_ascii_uppercase() {
                            if !name.is_empty() {
                                name.push(' ');
                            }
                            // Remove trailing punctuation from the last word
                            name.push_str(
                                word.trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?' || c == ':' || c == ';')
                            );
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if !name.is_empty() && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        search_start = abs_pos + 1;
    }

    names
}

/// Load all lore entries from a directory of Markdown files
pub fn load_lore_from_directory(dir: &Path) -> Result<Vec<LoreEntryResponse>> {
    let mut entries = Vec::new();

    if !dir.exists() {
        tracing::warn!("Lore directory does not exist: {:?}", dir);
        return Ok(entries);
    }

    let read_dir = std::fs::read_dir(dir)
        .map_err(|e| AppError::Io(format!("Failed to read lore directory: {}", e)))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "md" || ext == "markdown") {
            match parse_lore_file(&path) {
                Ok(Some(lore_entry)) => entries.push(lore_entry),
                Ok(None) => {} // skip files without valid frontmatter
                Err(e) => tracing::warn!("Failed to parse lore file {}: {}", path.display(), e),
            }
        }
    }

    entries.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(entries)
}

/// Parse Markdown content to HTML (for rendering in the frontend)
pub fn markdown_to_html(markdown: &str) -> String {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Create a temporary file with the given content and run parse_lore_file on it
    fn create_temp_md(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_story.md");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        (dir, path)
    }

    #[test]
    fn test_parse_yaml_frontmatter() {
        let content = "---\ntitle: War of the Spark\ntype: saga\nrelated_cards:\n  - Gideon\n  - Liliana\n---\n\n# Chapter 1\nThe story begins...";
        let (_dir, path) = create_temp_md(content);
        let result = parse_lore_file(&path).unwrap().unwrap();

        assert_eq!(result.title, "War of the Spark");
        assert_eq!(result.lore_type, "saga");
        assert!(result.content.contains("Chapter 1"));
        assert!(result.content.contains("The story begins"));
    }

    #[test]
    fn test_parse_toml_frontmatter() {
        let content = "title: Dominaria Lore\ntype: plane\nyear: 2020\n\n# Dominaria\nDominaria is a plane of vast history...";
        let (_dir, path) = create_temp_md(content);
        let result = parse_lore_file(&path).unwrap().unwrap();

        assert_eq!(result.title, "Dominaria Lore");
        assert_eq!(result.lore_type, "plane");
        assert!(result.content.contains("Dominaria is a plane"));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a Story\n\nNo frontmatter here.\n\nThe end.";
        let (_dir, path) = create_temp_md(content);
        let result = parse_lore_file(&path).unwrap().unwrap();

        assert_eq!(result.title, "test_story"); // falls back to filename
        assert_eq!(result.lore_type, "story"); // default
        assert!(result.content.contains("Just a Story"));
    }

    #[test]
    fn test_extract_card_names_brackets() {
        let body = "This card [[Black Lotus]] is powerful.\nAlso see [[Mox Emerald]] and [[Black Lotus]] again.";
        let names = extract_card_names_from_body(body);

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Black Lotus".to_string()));
        assert!(names.contains(&"Mox Emerald".to_string()));
    }

    #[test]
    fn test_extract_card_names_at_notation() {
        let body = "The planeswalker @Liliana Vess made a pact.\n@Gideon Jurgen fought bravely.";
        let names = extract_card_names_from_body(body);

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Liliana Vess".to_string()));
        assert!(names.contains(&"Gideon Jurgen".to_string()));
    }

    #[test]
    fn test_extract_card_names_combined() {
        let body = "[[Black Lotus]] and @Liliana Vess appear in this story.\nAlso @Gideon Jurgen.";
        let names = extract_card_names_from_body(body);

        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Black Lotus".to_string()));
        assert!(names.contains(&"Liliana Vess".to_string()));
        assert!(names.contains(&"Gideon Jurgen".to_string()));
    }

    #[test]
    fn test_parse_with_body_card_references() {
        let content = "---\ntitle: The Test Saga\ntype: saga\n---\n\n# Chapter 1\n\n[[Black Lotus]] was destroyed by @Liliana Vess.";
        let (_dir, path) = create_temp_md(content);
        let result = parse_lore_file(&path).unwrap().unwrap();

        assert_eq!(result.related_cards.len(), 2);
        assert!(result.related_cards.contains(&"Black Lotus".to_string()));
        assert!(result.related_cards.contains(&"Liliana Vess".to_string()));
    }

    #[test]
    fn test_markdown_to_html() {
        let md = "# Title\n\nThis is **bold** and *italic*.\n\n- List item 1\n- List item 2";
        let html = markdown_to_html(md);

        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<li>List item 1</li>"));
    }
}

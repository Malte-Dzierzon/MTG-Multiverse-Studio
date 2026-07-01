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

    // Try to extract YAML frontmatter (between --- delimiters)
    let (metadata, body) = if content.starts_with("---") {
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
            (None, content.as_str())
        }
    } else {
        (None, content.as_str())
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

    let related_cards: Vec<String> = metadata
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

    Ok(Some(LoreEntryResponse {
        id: 0,
        title,
        lore_type,
        content: body.to_string(),
        metadata: None, // metadata already embedded in content, skip duplicate
        related_cards,
    }))
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

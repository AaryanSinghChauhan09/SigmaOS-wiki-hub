//! SigmaOS wiki-hub — documentation hub for SigmaOS components.
//!
//! Builds a searchable index of component Wiki pages (markdown files),
//! grouped by category, with full-text search and JSON export.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Errors produced by this component.
#[derive(Debug, thiserror::Error)]
pub enum WikiHubError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("walk error: {0}")]
    Walk(String),
    #[error("not found: {0}")]
    NotFound(String),
}

/// A single indexed wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub title: String,
    pub path: PathBuf,
    pub category: String,
    pub content: String,
}

/// The wiki index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiIndex {
    #[serde(default)]
    pub pages: Vec<WikiPage>,
}

/// Extract title from first markdown heading, or filename.
fn extract_title(path: &Path, content: &str) -> String {
    for line in content.lines() {
        if let Some(t) = line.strip_prefix("# ") {
            return t.trim().to_string();
        }
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string()
}

/// Guess category from path components (first dir under root, else "general").
fn extract_category(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|p| p.components().next())
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("general")
        .to_string()
}

impl WikiIndex {
    /// Build an index by scanning a directory for .md files.
    pub fn build(root: &Path) -> Result<Self, WikiHubError> {
        let mut pages = Vec::new();
        for entry in walkdir::WalkDir::new(root).follow_links(false) {
            let entry = entry.map_err(|e| WikiHubError::Walk(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(path)?;
            pages.push(WikiPage {
                title: extract_title(path, &content),
                path: path.to_path_buf(),
                category: extract_category(root, path),
                content,
            });
        }
        pages.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(Self { pages })
    }

    /// Search pages by query (case-insensitive substring in title+content).
    pub fn search(&self, query: &str) -> Vec<&WikiPage> {
        let q = query.to_lowercase();
        self.pages
            .iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&q) || p.content.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Pages grouped by category.
    pub fn by_category(&self) -> BTreeMap<String, Vec<&WikiPage>> {
        let mut map: BTreeMap<String, Vec<&WikiPage>> = BTreeMap::new();
        for p in &self.pages {
            map.entry(p.category.clone()).or_default().push(p);
        }
        map
    }

    /// Get a page by title (exact match).
    pub fn get(&self, title: &str) -> Option<&WikiPage> {
        self.pages.iter().find(|p| p.title == title)
    }

    /// Render a markdown table of contents.
    pub fn toc(&self) -> String {
        let mut out = String::from("# SigmaOS Wiki Index\n\n");
        for (cat, pages) in self.by_category() {
            out.push_str(&format!("## {cat}\n\n"));
            for p in pages {
                out.push_str(&format!("- **{}** — {}\n", p.title, p.path.display()));
            }
            out.push('\n');
        }
        out
    }

    /// Save index (pages only, content included) as JSON.
    pub fn save(&self, path: &Path) -> Result<(), WikiHubError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Load index from JSON.
    pub fn load(path: &Path) -> Result<Self, WikiHubError> {
        let text = std::fs::read_to_string(path)
            .map_err(|_| WikiHubError::NotFound(path.display().to_string()))?;
        Ok(serde_json::from_str(&text)?)
    }
}

/// Default index path.
pub fn default_index_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("org", "sigmaos", "sigmaos")
        .map(|p| p.cache_dir().join("wiki-hub").join("index.json"))
}

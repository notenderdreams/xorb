use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Cli;
use crate::tree::FileNode;

pub struct ScannedFile {
    pub rel_path: PathBuf,
    pub content: Option<String>,
}

pub struct ScanResult {
    pub root_node: FileNode,
    pub files: Vec<ScannedFile>,
}

pub fn scan_directory(config: &Cli) -> Result<ScanResult> {
    let path = &config.path;
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    if !canonical.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    if canonical.is_file() {
        let file_name = canonical
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        let rel_path = PathBuf::from(&file_name);
        let max_bytes = config.max_size_kb * 1024;
        let (is_binary, content) = read_file_content(&canonical, max_bytes);

        let root_node = FileNode::File {
            name: file_name,
            rel_path: rel_path.clone(),
            is_binary,
        };

        return Ok(ScanResult {
            root_node,
            files: vec![ScannedFile { rel_path, content }],
        });
    }

    let mut walker = WalkBuilder::new(&canonical);
    walker
        .hidden(!config.hidden)
        .standard_filters(!config.no_ignore);

    let max_bytes = config.max_size_kb * 1024;
    let mut dir_entries: HashMap<PathBuf, Vec<FileNode>> = HashMap::new();
    let mut scanned_files = Vec::new();

    dir_entries.insert(PathBuf::new(), Vec::new());

    for entry in walker.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_path = entry.path();
        if entry_path == canonical {
            continue;
        }

        let rel_path = match entry_path.strip_prefix(&canonical) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => continue,
        };

        let file_name = match rel_path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        let parent_rel = rel_path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

        if entry_path.is_dir() {
            dir_entries.entry(rel_path.clone()).or_default();
        } else if entry_path.is_file() {
            let (is_binary, content) = read_file_content(entry_path, max_bytes);

            let file_node = FileNode::File {
                name: file_name,
                rel_path: rel_path.clone(),
                is_binary,
            };

            // Ensure parent dirs up to root exist in map
            let mut current = parent_rel.as_path();
            while !current.as_os_str().is_empty() {
                dir_entries.entry(current.to_path_buf()).or_default();
                current = current.parent().unwrap_or_else(|| Path::new(""));
            }

            dir_entries.entry(parent_rel).or_default().push(file_node);
            scanned_files.push(ScannedFile { rel_path, content });
        }
    }

    let root_node = build_node_tree(Path::new(""), &mut dir_entries);

    // Sort files by path for deterministic output
    scanned_files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    Ok(ScanResult {
        root_node,
        files: scanned_files,
    })
}

fn read_file_content(path: &Path, max_bytes: u64) -> (bool, Option<String>) {
    if max_bytes > 0 && fs::metadata(path).is_ok_and(|m| m.len() > max_bytes) {
        return (true, None);
    }

    match fs::read(path) {
        Ok(bytes) => {
            if bytes.contains(&0) {
                (true, None)
            } else {
                match String::from_utf8(bytes) {
                    Ok(text) => (false, Some(text)),
                    Err(_) => (true, None),
                }
            }
        }
        Err(_) => (true, None),
    }
}

fn build_node_tree(
    rel_path: &Path,
    dir_entries: &mut HashMap<PathBuf, Vec<FileNode>>,
) -> FileNode {
    let mut children = dir_entries.remove(rel_path).unwrap_or_default();

    let immediate_subdirs: Vec<PathBuf> = dir_entries
        .keys()
        .filter(|k| k.parent() == Some(rel_path) && !k.as_os_str().is_empty())
        .cloned()
        .collect();

    for subdir in immediate_subdirs {
        let child_node = build_node_tree(&subdir, dir_entries);
        children.push(child_node);
    }

    let dir_name = if rel_path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        rel_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    };

    let mut node = FileNode::Directory {
        name: dir_name,
        rel_path: rel_path.to_path_buf(),
        children,
    };

    node.sort_children();
    node
}

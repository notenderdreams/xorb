use anyhow::{Context, Result};
use ignore::overrides::OverrideBuilder;
use ignore::{WalkBuilder, WalkState};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;

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

enum ScannedItem {
    Dir(PathBuf),
    File {
        rel_path: PathBuf,
        node: FileNode,
        scanned: ScannedFile,
    },
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

    if let Some(depth) = config.max_depth {
        walker.max_depth(Some(depth));
    }

    // Configure include and exclude glob patterns
    if config.include.is_some() || config.exclude.is_some() {
        let mut ov_builder = OverrideBuilder::new(&canonical);

        if let Some(includes) = &config.include {
            for pat in includes {
                let pattern_str = if pat.starts_with('/') || pat.starts_with('*') {
                    pat.clone()
                } else {
                    format!("**/{}", pat)
                };
                let _ = ov_builder.add(&pattern_str);
            }
        }

        if let Some(excludes) = &config.exclude {
            for pat in excludes {
                let pattern_str = if pat.starts_with('!') {
                    pat.clone()
                } else {
                    format!("!{}", pat)
                };
                let _ = ov_builder.add(&pattern_str);
            }
        }

        if let Ok(overrides) = ov_builder.build() {
            walker.overrides(overrides);
        }
    }

    // Git diff filtering if enabled
    let git_diff_set = if let Some(ref git_ref) = config.diff {
        Some(get_git_diff_files(&canonical, git_ref)?)
    } else {
        None
    };

    let max_bytes = config.max_size_kb * 1024;
    let (tx, rx) = mpsc::channel();

    let root = canonical.clone();
    walker.build_parallel().run(|| {
        let tx = tx.clone();
        let root = root.clone();
        let git_diff_set = git_diff_set.clone();

        Box::new(move |entry| {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            let entry_path = entry.path();
            if entry_path == root {
                return WalkState::Continue;
            }

            let rel_path = match entry_path.strip_prefix(&root) {
                Ok(rel) => rel.to_path_buf(),
                Err(_) => return WalkState::Continue,
            };

            let file_name = match rel_path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => return WalkState::Continue,
            };

            if entry_path.is_dir() {
                let _ = tx.send(ScannedItem::Dir(rel_path));
            } else if entry_path.is_file() {
                // If diff filtering is active, skip files not present in diff set
                if let Some(ref diff_files) = git_diff_set
                    && !diff_files.contains(&rel_path)
                {
                    return WalkState::Continue;
                }

                let (is_binary, content) = read_file_content(entry_path, max_bytes);

                let node = FileNode::File {
                    name: file_name,
                    rel_path: rel_path.clone(),
                    is_binary,
                };

                let scanned = ScannedFile {
                    rel_path: rel_path.clone(),
                    content,
                };

                let _ = tx.send(ScannedItem::File {
                    rel_path,
                    node,
                    scanned,
                });
            }

            WalkState::Continue
        })
    });

    drop(tx);

    let mut dir_entries: HashMap<PathBuf, Vec<FileNode>> = HashMap::new();
    let mut scanned_files = Vec::new();

    dir_entries.insert(PathBuf::new(), Vec::new());

    for item in rx {
        match item {
            ScannedItem::Dir(rel_path) => {
                dir_entries.entry(rel_path).or_default();
            }
            ScannedItem::File {
                rel_path,
                node,
                scanned,
            } => {
                let parent_rel = rel_path
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .to_path_buf();

                // Ensure parent dirs up to root exist in map
                let mut current = parent_rel.as_path();
                while !current.as_os_str().is_empty() {
                    dir_entries.entry(current.to_path_buf()).or_default();
                    current = current.parent().unwrap_or_else(|| Path::new(""));
                }

                dir_entries.entry(parent_rel).or_default().push(node);
                scanned_files.push(scanned);
            }
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

fn get_git_diff_files(root: &Path, git_ref: &str) -> Result<HashSet<PathBuf>> {
    let mut set = HashSet::new();

    // 1. Get diff against git_ref
    let mut cmd = Command::new("git");
    cmd.current_dir(root).arg("diff").arg("--name-only");

    if !git_ref.is_empty() {
        cmd.arg(git_ref);
    }

    if let Ok(output) = cmd.output()
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                set.insert(PathBuf::from(trimmed));
            }
        }
    }

    // 2. Get untracked or staged working tree files
    let mut status_cmd = Command::new("git");
    status_cmd
        .current_dir(root)
        .arg("status")
        .arg("--porcelain");

    if let Ok(output) = status_cmd.output()
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.len() > 3 {
                let path_str = trimmed[3..].trim_matches('"');
                set.insert(PathBuf::from(path_str));
            }
        }
    }

    Ok(set)
}

fn read_file_content(path: &Path, max_bytes: u64) -> (bool, Option<String>) {
    if max_bytes > 0 && fs::metadata(path).is_ok_and(|m| m.len() > max_bytes) {
        return (true, None);
    }

    match fs::read(path) {
        Ok(bytes) => {
            if memchr::memchr(0, &bytes).is_some() {
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

fn build_node_tree(rel_path: &Path, dir_entries: &mut HashMap<PathBuf, Vec<FileNode>>) -> FileNode {
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

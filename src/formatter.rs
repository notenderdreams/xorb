use crate::scanner::ScanResult;
use std::path::Path;

pub fn format_bundle(scan_result: &ScanResult) -> String {
    let mut output = String::new();

    // Repository Structure section
    output.push_str("## Repository Structure\n\n```\n");
    let tree_str = scan_result.root_node.render_tree();
    output.push_str(&tree_str);
    output.push_str("```\n\n");

    // Files section
    output.push_str("## Files\n\n");

    for scanned in &scan_result.files {
        let path_display = scanned.rel_path.display();
        let lang = detect_language(&scanned.rel_path);

        match &scanned.content {
            Some(text) => {
                output.push_str(&format!(
                    "# File: {}\n```{}\n{}\n```\n\n",
                    path_display,
                    lang,
                    text.trim_end()
                ));
            }
            None => {
                output.push_str(&format!(
                    "# File: {} (binary or skipped)\n```\n<skipped>\n```\n\n",
                    path_display
                ));
            }
        }
    }

    output
}

pub fn detect_language(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "rust",
        Some("js") | Some("mjs") | Some("cjs") => "javascript",
        Some("ts") | Some("mts") | Some("cts") => "typescript",
        Some("jsx") => "jsx",
        Some("tsx") => "tsx",
        Some("py") => "python",
        Some("go") => "go",
        Some("c") | Some("h") => "c",
        Some("cpp") | Some("hpp") | Some("cc") | Some("cxx") => "cpp",
        Some("java") => "java",
        Some("kt") | Some("kts") => "kotlin",
        Some("swift") => "swift",
        Some("rb") => "ruby",
        Some("php") => "php",
        Some("cs") => "csharp",
        Some("json") => "json",
        Some("yaml") | Some("yml") => "yaml",
        Some("toml") => "toml",
        Some("md") => "markdown",
        Some("sh") | Some("bash") | Some("zsh") => "bash",
        Some("html") | Some("htm") => "html",
        Some("css") | Some("scss") | Some("sass") => "css",
        Some("sql") => "sql",
        Some("xml") | Some("svg") => "xml",
        Some("proto") => "protobuf",
        _ => "",
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    // Rough heuristic: ~4 characters per token
    text.len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::ScannedFile;
    use crate::tree::FileNode;
    use std::path::PathBuf;

    #[test]
    fn test_language_detection() {
        assert_eq!(detect_language(Path::new("main.rs")), "rust");
        assert_eq!(detect_language(Path::new("app.tsx")), "tsx");
        assert_eq!(detect_language(Path::new("Cargo.toml")), "toml");
        assert_eq!(detect_language(Path::new("unknown.xyz")), "");
    }

    #[test]
    fn test_format_bundle() {
        let root_node = FileNode::Directory {
            name: ".".to_string(),
            rel_path: PathBuf::from("."),
            children: vec![FileNode::File {
                name: "main.rs".to_string(),
                rel_path: PathBuf::from("main.rs"),
                is_binary: false,
            }],
        };

        let files = vec![ScannedFile {
            rel_path: PathBuf::from("main.rs"),
            content: Some("fn main() {}".to_string()),
        }];

        let result = ScanResult { root_node, files };
        let formatted = format_bundle(&result);

        assert!(formatted.contains("## Repository Structure"));
        assert!(formatted.contains("└── main.rs"));
        assert!(formatted.contains("## Files"));
        assert!(formatted.contains("# File: main.rs\n```rust\nfn main() {}\n```"));
    }
}

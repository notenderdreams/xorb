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

/// Accurately estimates LLM token count for source code and markdown documents
/// without external tokenizer dependencies.
///
/// Accounts for:
/// - Sub-word tokens in identifier names (camelCase, snake_case, PascalCase)
/// - Punctuation and special code symbols (operators, brackets, indentation)
/// - Numbers, string literals, whitespace, and newline overhead
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let mut count: f64 = 0.0;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\n' => {
                count += 1.0;
            }
            ' ' | '\t' => {
                // Leading/consecutive whitespace in code tokenizes per 3-4 spaces in BPE
                let mut space_run = 1;
                while let Some(&next_c) = chars.peek() {
                    if next_c == ' ' || next_c == '\t' {
                        space_run += 1;
                        chars.next();
                    } else {
                        break;
                    }
                }
                count += (space_run as f64 / 4.0).ceil();
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                ident.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_alphanumeric() || next_c == '_' {
                        ident.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                count += count_identifier_tokens(&ident);
            }
            '0'..='9' => {
                let mut num_len = 1;
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() || next_c == '.' || next_c == '_' {
                        num_len += 1;
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Short numbers = 1 token, longer numbers ~ 1 token per 2-3 digits
                count += (num_len as f64 / 2.5).ceil().max(1.0);
            }
            // Punctuation and code syntax symbols (braces, parens, operators, quotes)
            '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '.' | ':' | '=' | '+' | '-' | '*'
            | '/' | '<' | '>' | '!' | '&' | '|' | '^' | '%' | '~' | '?' | '"' | '\'' | '`'
            | '#' | '@' | '$' | '\\' => {
                count += 1.0;
            }
            _ => {
                // Non-ASCII or multi-byte unicode characters (e.g. emojis, non-English)
                let len = c.len_utf8();
                if len > 1 {
                    count += (len as f64 / 2.0).ceil();
                } else {
                    count += 1.0;
                }
            }
        }
    }

    count.round() as usize
}

/// Helper to estimate sub-word BPE tokenization of code identifiers (snake_case, camelCase, PascalCase)
fn count_identifier_tokens(ident: &str) -> f64 {
    if ident.is_empty() {
        return 0.0;
    }

    // Identifiers up to 6 characters are usually 1 token in BPE vocabularies
    if ident.len() <= 6 {
        return 1.0;
    }

    let parts: Vec<&str> = ident.split('_').collect();
    if parts.is_empty() {
        return 1.0;
    }

    let mut total_tokens = 0.0;
    let num_parts = parts.len();
    for (idx, part) in parts.into_iter().enumerate() {
        if part.is_empty() {
            // Consecutive or leading underscores are 1 token each
            total_tokens += 1.0;
            continue;
        }

        let mut subwords = 0;
        let mut current_len = 0;
        let chars: Vec<char> = part.chars().collect();

        for i in 0..chars.len() {
            let ch = chars[i];
            let is_upper = ch.is_ascii_uppercase();
            let prev_lower = if i > 0 {
                chars[i - 1].is_ascii_lowercase()
            } else {
                false
            };

            if is_upper && prev_lower {
                if current_len > 0 {
                    subwords += (current_len as f64 / 4.0).ceil() as usize;
                    current_len = 0;
                }
                current_len += 1;
            } else {
                current_len += 1;
            }
        }

        if current_len > 0 {
            subwords += (current_len as f64 / 4.0).ceil() as usize;
        }

        total_tokens += (subwords as f64).max(1.0);
        // Underscore separator between parts is 1 token in BPE tokenizers
        if idx < num_parts - 1 {
            total_tokens += 1.0;
        }
    }

    total_tokens
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

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);

        let rust_code = "fn calculate_total_price(item_count: usize, tax_rate: f64) -> f64 {\n    item_count as f64 * tax_rate\n}";
        let count = estimate_tokens(rust_code);
        // Expecting ~35-50 tokens for this function snippet with identifiers, types, operators, and spaces
        assert!(count >= 30 && count <= 55, "Token count {} out of expected range", count);
    }
}

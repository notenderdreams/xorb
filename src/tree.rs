use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum FileNode {
    File {
        name: String,
        rel_path: PathBuf,
        is_binary: bool,
    },
    Directory {
        name: String,
        rel_path: PathBuf,
        children: Vec<FileNode>,
    },
}

impl FileNode {
    pub fn name(&self) -> &str {
        match self {
            FileNode::File { name, .. } => name,
            FileNode::Directory { name, .. } => name,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, FileNode::Directory { .. })
    }

    pub fn sort_children(&mut self) {
        if let FileNode::Directory { children, .. } = self {
            for child in children.iter_mut() {
                child.sort_children();
            }
            children.sort_by(|a, b| match (a.is_dir(), b.is_dir()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name().cmp(b.name()),
            });
        }
    }

    pub fn render_tree(&self) -> String {
        let mut out = String::new();
        match self {
            FileNode::Directory { children, .. } => {
                for (i, child) in children.iter().enumerate() {
                    let is_last = i == children.len() - 1;
                    child.render_node("", is_last, &mut out);
                }
            }
            FileNode::File { name, .. } => {
                out.push_str(name);
                out.push('\n');
            }
        }
        out
    }

    fn render_node(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        match self {
            FileNode::Directory { name, children, .. } => {
                out.push_str(&format!("{}{}{}/\n", prefix, connector, name));
                let new_prefix = format!("{}{}", prefix, child_prefix);
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == children.len() - 1;
                    child.render_node(&new_prefix, child_is_last, out);
                }
            }
            FileNode::File { name, .. } => {
                out.push_str(&format!("{}{}{}\n", prefix, connector, name));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_rendering_and_sorting() {
        let mut root = FileNode::Directory {
            name: ".".to_string(),
            rel_path: PathBuf::from("."),
            children: vec![
                FileNode::File {
                    name: "b.txt".to_string(),
                    rel_path: PathBuf::from("b.txt"),
                    is_binary: false,
                },
                FileNode::Directory {
                    name: "src".to_string(),
                    rel_path: PathBuf::from("src"),
                    children: vec![
                        FileNode::File {
                            name: "main.rs".to_string(),
                            rel_path: PathBuf::from("src/main.rs"),
                            is_binary: false,
                        },
                        FileNode::File {
                            name: "lib.rs".to_string(),
                            rel_path: PathBuf::from("src/lib.rs"),
                            is_binary: false,
                        },
                    ],
                },
                FileNode::File {
                    name: "a.txt".to_string(),
                    rel_path: PathBuf::from("a.txt"),
                    is_binary: false,
                },
            ],
        };

        root.sort_children();

        let rendered = root.render_tree();
        let expected = "\
├── src/
│   ├── lib.rs
│   └── main.rs
├── a.txt
└── b.txt
";
        assert_eq!(rendered, expected);
    }
}

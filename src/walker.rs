use crate::cli::Args;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: Vec<Entry>,
}

impl Entry {
    pub fn file(name: String, size: u64) -> Self {
        Entry {
            name,
            is_dir: false,
            size,
            children: Vec::new(),
        }
    }

    pub fn dir(name: String, children: Vec<Entry>) -> Self {
        Entry {
            name,
            is_dir: true,
            size: 0,
            children,
        }
    }
}

pub fn walk(orig_root: &Path, args: &Args) -> std::io::Result<Entry> {
    let abs_root = std::path::absolute(orig_root)
        .unwrap_or_else(|_| orig_root.to_path_buf());

    let gitignore = if !args.no_gitignore {
        build_gitignore(&abs_root)
    } else {
        Gitignore::empty()
    };

    let root_name = orig_root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| orig_root.display().to_string());

    // 把 gitignore 的根目录也传下去，方便计算相对路径
    let children = walk_children(&abs_root, &abs_root, args, 0, &gitignore)?;
    Ok(Entry::dir(root_name, children))
}

fn build_gitignore(root: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(root);

    let gi = root.join(".gitignore");
    if gi.is_file() {
        let _ = builder.add(&gi);
    }

    let exclude = root.join(".git/info/exclude");
    if exclude.is_file() {
        let _ = builder.add(&exclude);
    }

    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

fn walk_children(
    dir: &Path,
    root: &Path,
    args: &Args,
    depth: usize,
    gitignore: &Gitignore,
) -> std::io::Result<Vec<Entry>> {
    if args.depth != 0 && depth >= args.depth {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| {
        let is_dir = e.path().is_dir();
        let name = e.file_name().to_string_lossy().to_string();
        (!is_dir, name)
    });

    let mut children = Vec::new();

    for entry in entries {
        let entry_name = entry.file_name().to_string_lossy().to_string();
        let entry_path = entry.path();
        let is_dir = entry_path.is_dir();

        if args.should_ignore(&entry_name) {
            continue;
        }

        if !args.no_gitignore {
            // 关键：传相对于 root 的路径给 gitignore
            if let Ok(rel) = entry_path.strip_prefix(root) {
                let matched = gitignore.matched_path_or_any_parents(rel, is_dir);
                if matched.is_ignore() {
                    continue;
                }
            }
        }

        let meta = entry.metadata()?;

        if is_dir {
            let sub = walk_children(&entry_path, root, args, depth + 1, gitignore)?;
            children.push(Entry::dir(entry_name, sub));
        } else if args.match_ext(&entry_name) {
            children.push(Entry::file(entry_name, meta.len()));
        }
    }

    Ok(children)
}
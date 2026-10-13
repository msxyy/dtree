use crate::cli::Args;
use crate::walker::Entry;
use colored::*;

/// 根据 args.format 选择渲染方式
pub fn render(entry: &Entry, args: &Args) -> String {
    if args.is_markdown() {
        render_markdown(entry, args.size)
    } else {
        render_tree(entry, args.use_color(), args.ascii, args.size)
    }
}

// ---------- 树形格式 ----------

pub fn render_tree(entry: &Entry, color: bool, ascii: bool, show_size: bool) -> String {
    let size_width = if show_size { max_size_width(entry) } else { 0 };
    let mut out = String::new();

    let root_suffix = if entry.is_dir { "/" } else { "" };
    let root_line = format!("{}{}", entry.name, root_suffix);
    let root_size = if show_size && !entry.is_dir {
        format!("  {:>w$}", format_size(entry.size), w = size_width)
    } else {
        String::new()
    };

    if color && entry.is_dir {
        out.push_str(&format!("{}{}\n", root_line.blue().bold(), root_size));
    } else {
        out.push_str(&format!("{}{}\n", root_line, root_size));
    }

    let last = entry.children.len().saturating_sub(1);
    for (i, child) in entry.children.iter().enumerate() {
        render_tree_child(
            child,
            "",
            i == last,
            color,
            ascii,
            show_size,
            size_width,
            &mut out,
        );
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn render_tree_child(
    entry: &Entry,
    prefix: &str,
    is_last: bool,
    color: bool,
    ascii: bool,
    show_size: bool,
    size_width: usize,
    out: &mut String,
) {
    let (connector, next_vert) = if ascii {
        if is_last {
            ("`-- ", "    ")
        } else {
            ("|-- ", "|   ")
        }
    } else if is_last {
        ("└── ", "    ")
    } else {
        ("├── ", "│   ")
    };

    let suffix = if entry.is_dir { "/" } else { "" };
    let name = format!("{}{}", entry.name, suffix);

    let size_part = if show_size && !entry.is_dir {
        format!("  {:>w$}", format_size(entry.size), w = size_width)
    } else {
        String::new()
    };

    let line = if color {
        let c = if entry.is_dir {
            name.blue().bold()
        } else {
            name.normal()
        };
        format!(
            "{}{}{}{}",
            prefix.bright_black(),
            connector.bright_black(),
            c,
            size_part
        )
    } else {
        format!("{}{}{}{}", prefix, connector, name, size_part)
    };
    out.push_str(&line);
    out.push('\n');

    let child_prefix = format!("{}{}", prefix, next_vert);

    let last = entry.children.len().saturating_sub(1);
    for (i, child) in entry.children.iter().enumerate() {
        render_tree_child(
            child,
            &child_prefix,
            i == last,
            color,
            ascii,
            show_size,
            size_width,
            out,
        );
    }
}

fn max_size_width(entry: &Entry) -> usize {
    fn rec(e: &Entry, max: &mut usize) {
        if !e.is_dir {
            let len = format_size(e.size).len();
            if len > *max {
                *max = len;
            }
        }
        for c in &e.children {
            rec(c, max);
        }
    }
    let mut max = 0;
    rec(entry, &mut max);
    max
}

// ---------- Markdown 格式 ----------

pub fn render_markdown(entry: &Entry, show_size: bool) -> String {
    let mut out = String::new();

    let root_suffix = if entry.is_dir { "/" } else { "" };
    let root_size = if show_size && !entry.is_dir {
        format!(" ({})", format_size(entry.size))
    } else {
        String::new()
    };
    out.push_str(&format!("- {}{}{}\n", entry.name, root_suffix, root_size));

    for child in &entry.children {
        render_md_child(child, 1, show_size, &mut out);
    }

    out
}

fn render_md_child(entry: &Entry, depth: usize, show_size: bool, out: &mut String) {
    let indent = "  ".repeat(depth);
    let suffix = if entry.is_dir { "/" } else { "" };
    let size_str = if show_size && !entry.is_dir {
        format!(" ({})", format_size(entry.size))
    } else {
        String::new()
    };
    out.push_str(&format!(
        "{}- {}{}{}\n",
        indent, entry.name, suffix, size_str
    ));

    for child in &entry.children {
        render_md_child(child, depth + 1, show_size, out);
    }
}

// ---------- 统计信息 ----------

pub struct Stats {
    pub dirs: usize,
    pub files: usize,
    pub total_size: u64,
}

pub fn compute_stats(entry: &Entry) -> Stats {
    let mut stats = Stats {
        dirs: 0,
        files: 0,
        total_size: 0,
    };
    walk_stats(entry, &mut stats);
    stats
}

fn walk_stats(entry: &Entry, stats: &mut Stats) {
    if entry.is_dir {
        stats.dirs += 1;
    } else {
        stats.files += 1;
        stats.total_size += entry.size;
    }
    for child in &entry.children {
        walk_stats(child, stats);
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn stats_text(stats: &Stats) -> String {
    let mut out = String::new();
    out.push_str("\n---\n");
    out.push_str(&format!("目录: {}\n", stats.dirs));
    out.push_str(&format!("文件: {}\n", stats.files));
    out.push_str(&format!("总大小: {}\n", format_size(stats.total_size)));
    out
}
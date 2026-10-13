mod cli;
mod render;
mod walker;

use clap::Parser;
use colored::control;
use std::path::Path;

/// Windows 下：启用 UTF-8 和 VT 处理
#[cfg(windows)]
fn setup_console() {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleCP, SetConsoleMode,
        SetConsoleOutputCP, ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
    };

    unsafe {
        SetConsoleOutputCP(65001);
        SetConsoleCP(65001);

        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

#[cfg(not(windows))]
fn setup_console() {}

fn main() {
    setup_console();

    let args = cli::Args::parse();

    if args.no_color || args.is_markdown() {
        control::set_override(false);
    }

    let root = Path::new(&args.path);
    if !root.exists() {
        eprintln!("路径不存在: {}", args.path);
        std::process::exit(1);
    }

    let tree = match walker::walk(root, &args) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("遍历出错: {}", e);
            std::process::exit(1);
        }
    };

    let mut text = render::render(&tree, &args);

    if args.stats {
        let stats = render::compute_stats(&tree);
        text.push_str(&render::stats_text(&stats));
    }

    let mut did_something = false;

    if let Some(path) = &args.output {
        let mut bytes = Vec::with_capacity(text.len() + 3);
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        bytes.extend_from_slice(text.as_bytes());

        match std::fs::write(path, bytes) {
            Ok(_) => {
                eprintln!("已写入文件: {}", path);
                did_something = true;
            }
            Err(e) => {
                eprintln!("写入文件失败: {}", e);
                std::process::exit(1);
            }
        }
    }

    if args.copy {
        let plain = if args.use_color() && !args.is_markdown() {
            let mut plain_args = cli::Args::parse_from(std::env::args());
            plain_args.no_color = true;
            let mut plain_text = render::render(&tree, &plain_args);
            if args.stats {
                let stats = render::compute_stats(&tree);
                plain_text.push_str(&render::stats_text(&stats));
            }
            plain_text
        } else {
            text.clone()
        };

        match arboard::Clipboard::new().and_then(|mut c| c.set_text(plain)) {
            Ok(_) => {
                eprintln!("已复制到剪贴板");
                did_something = true;
            }
            Err(e) => {
                eprintln!("复制到剪贴板失败: {}", e);
                std::process::exit(1);
            }
        }
    }

    if !did_something || args.print {
        print!("{}", text);
    }
}
use clap::Parser;

/// 目录树查看器
#[derive(Parser, Debug)]
#[command(name = "dtree", version, about = "以树状展示目录结构")]
pub struct Args {
    /// 要展开的目录，默认当前目录
    #[arg(default_value = ".")]
    pub path: String,

    /// 最大深度，0 表示不限制
    #[arg(short, long, default_value_t = 0)]
    pub depth: usize,

    /// 额外忽略的目录或文件名，逗号分隔（追加到默认列表）
    #[arg(short, long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// 不使用默认忽略列表，只用 --ignore 指定的
    #[arg(long, default_value_t = false)]
    pub no_default_ignore: bool,

    /// 不使用 .gitignore 规则（默认使用）
    #[arg(long, default_value_t = false)]
    pub no_gitignore: bool,

    /// 只显示这些扩展名，逗号分隔，如 rs,toml
    #[arg(short, long, value_delimiter = ',')]
    pub ext: Vec<String>,

    /// 是否显示隐藏文件（以 . 开头）
    #[arg(short = 'a', long, default_value_t = false)]
    pub all: bool,

    /// 复制结果到剪贴板
    #[arg(short = 'c', long, default_value_t = false)]
    pub copy: bool,

    /// 把结果写入指定文件
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// 即使使用了 --copy 或 --output，也同时在终端打印
    #[arg(short = 'p', long, default_value_t = false)]
    pub print: bool,

    /// 输出格式：tree（默认）、md/markdown
    #[arg(short = 'f', long, default_value = "tree")]
    pub format: String,

    /// 关闭彩色输出
    #[arg(long, default_value_t = false)]
    pub no_color: bool,

    /// 显示统计信息（目录数、文件数、总大小）
    #[arg(short = 's', long, default_value_t = false)]
    pub stats: bool,

    /// 显示每个文件的大小
    #[arg(long, default_value_t = false)]
    pub size: bool,

    /// 用 ASCII 符号代替 Unicode 树形符号（兼容老旧终端）
    #[arg(long, default_value_t = false)]
    pub ascii: bool,
}

fn default_ignores() -> Vec<String> {
    vec![
        "target".to_string(),
        ".git".to_string(),
        "node_modules".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
        "tree.txt".to_string(),
        "tree.md".to_string(),
    ]
}

impl Args {
    pub fn effective_ignores(&self) -> Vec<String> {
        let mut list = if self.no_default_ignore {
            Vec::new()
        } else {
            default_ignores()
        };
        list.extend(self.ignore.iter().cloned());
        list
    }

    pub fn should_ignore(&self, name: &str) -> bool {
        if !self.all && name.starts_with('.') && name != "." {
            return true;
        }
        self.effective_ignores().iter().any(|i| i == name)
    }

    pub fn match_ext(&self, name: &str) -> bool {
        if self.ext.is_empty() {
            return true;
        }
        match name.rsplit_once('.') {
            Some((_, ext)) => self.ext.iter().any(|e| e == ext),
            None => false,
        }
    }

    pub fn is_markdown(&self) -> bool {
        matches!(self.format.as_str(), "md" | "markdown")
    }

    pub fn use_color(&self) -> bool {
        !self.no_color
    }
}
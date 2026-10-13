# dtree

以树状展示目录结构，支持 `.gitignore` 自动忽略、复制到剪贴板、Markdown 输出、文件大小统计。

比 Windows 自带的 `tree /F` 更干净、更好用。

## 特性

- 默认忽略 `target`、`node_modules`、`.git` 等大目录
- 自动读取 `.gitignore` 规则
- 支持深度限制、扩展名过滤、自定义忽略
- 一键复制到剪贴板，无颜色转义码
- 输出 Markdown，直接粘到 README
- 显示文件大小、目录数、文件数、总大小
- Windows 下自动启用 UTF-8 和 ANSI 颜色
- 彩色输出，支持 `--ascii` 兼容模式

## 安装

### 从源码编译

```bash
git clone https://github.com/你的用户名/dtree.git
cd dtree
cargo install --path .
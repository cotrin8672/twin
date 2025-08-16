# Cargo (crates.io) 公開設定

## 1. Cargo.tomlの準備

```toml
[package]
name = "twin-cli"  # "twin"は既に取られている可能性があるため
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Git worktree wrapper with side effects (symlinks and hooks)"
repository = "https://github.com/yourusername/twin"
homepage = "https://github.com/yourusername/twin"
documentation = "https://docs.rs/twin-cli"
readme = "README.md"
keywords = ["git", "worktree", "cli", "symlink", "hooks"]
categories = ["command-line-utilities", "development-tools"]
license = "MIT"

# バイナリ名を指定
[[bin]]
name = "twin"
path = "src/main.rs"

# 除外ファイル
exclude = [
    ".github/*",
    "tests/*",
    "docs/*",
    ".gitignore",
    ".twin.toml",
]
```

## 2. crates.ioアカウント設定

```bash
# アカウント作成後、APIトークンを取得
cargo login <your-api-token>
```

## 3. 公開前チェック

```bash
# ドライラン（実際には公開しない）
cargo publish --dry-run

# パッケージ内容の確認
cargo package --list
```

## 4. 公開

```bash
cargo publish
```

## 5. GitHub Actionsでの自動公開

```yaml
- name: Publish to crates.io
  if: startsWith(github.ref, 'refs/tags/v')
  env:
    CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
  run: cargo publish
```
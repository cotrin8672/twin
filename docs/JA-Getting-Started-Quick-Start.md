# クイックスタートガイド

[English](Getting-Started-Quick-Start) | **日本語**

5分で Twin を起動して実行します。

## インストール

### ソースから
```bash
# クローンしてビルド
git clone https://github.com/your-org/twin.git
cd twin
cargo install --path .

# インストールを確認
twin --version
```

### Cargo から（将来）
```bash
cargo install twin
```

## 基本的な使い方

### 1. プロジェクトを初期化
Git リポジトリに移動：
```bash
cd your-project
```

### 2. 最初の環境を作成
```bash
# 新しいワークツリー環境を作成
twin create feature-authentication

# 作成されたパスが出力されます
# ../feature-authentication
```

### 3. 環境をリスト表示
```bash
twin list

# 出力：
# ┌──────────────────────┬─────────────────────────┬────────┐
# │ Branch               │ Path                    │ Prunable│
# ├──────────────────────┼─────────────────────────┼────────┤
# │ feature-authentication│ ../feature-authentication│ false  │
# └──────────────────────┴─────────────────────────┴────────┘
```

### 4. 環境に移動
```bash
# cd コマンドを取得
twin create another-feature --cd-command
# 出力: cd "../another-feature"

# またはパスだけを取得
twin create test-branch --print-path
# 出力: ../test-branch
```

### 5. 環境を削除
```bash
twin remove feature-authentication
```

## 設定例

プロジェクトルートに `twin.toml` ファイルを作成：
```toml
# ワークツリーのベースディレクトリを設定
worktree_base = "../workspaces"

# ファイルマッピングを定義
[[files]]
source = ".env.template"
target = ".env"
mapping_type = "copy"
skip_if_exists = true
description = "環境変数"

[[files]]
source = ".claude/config.json"
target = ".claude/config.json"
mapping_type = "symlink"
description = "Claude 設定"
```

ソース: [README.md#L27-67](https://github.com/your-org/twin/blob/main/README.md#L27-67)
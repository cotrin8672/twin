# Twin - Git Worktree Wrapper with Side Effects

Twin は Git worktree の薄いラッパーで、シンボリックリンクやフックなどの副作用を追加した CLI ツールです。

## 概要

Git worktree コマンドをベースに、以下の副作用を追加しました：
- シンボリックリンクの自動作成・削除
- worktree 作成・削除時のフック実行
- 設定ファイルによるカスタマイズ

## 主な機能

- **Worktree管理**: Git worktreeのラッパーとして動作
- **シンボリックリンク**: 設定ファイルの共有と環境間での一貫性維持
- **クロスプラットフォーム**: Windows/macOS/Linuxに対応
- **フック機能**: worktree作成・削除時のカスタムスクリプト実行
- **柔軟な設定**: TOML形式の設定ファイルによるカスタマイズ

## インストール

### Cargo (推奨)
```bash
# crates.ioから直接インストール
cargo install twin-cli

# または、最新の開発版をGitHubからインストール
cargo install --git https://github.com/yourusername/twin
```

### ソースからビルド
```bash
# リポジトリをクローン
git clone https://github.com/yourusername/twin
cd twin

# ビルド＆インストール
cargo install --path .
```

### バイナリダウンロード
[Releases](https://github.com/yourusername/twin/releases)ページから、お使いのプラットフォーム用のバイナリをダウンロードできます。

## 使用方法

### 基本コマンド

#### Worktreeの作成（git worktree add のラッパー）
```bash
# 基本的な使用方法
twin add <path> [<branch>]

# 新しいブランチを作成
twin add ../feature-new -b feature-new

# 既存のブランチをチェックアウト
twin add ../hotfix hotfix-branch

# 設定ファイルを指定（副作用を適用）
twin add ../feature --config .twin.toml

# Git worktree のみ実行（副作用をスキップ）
twin add ../feature --git-only

# その他の git worktree オプションもサポート
twin add ../feature --detach
twin add ../feature --lock
```

#### Worktreeの一覧表示（git worktree list のラッパー）
```bash
# デフォルト（テーブル形式）
twin list

# JSON形式で出力
twin list --format json

# シンプルな形式
twin list --format simple
```

#### Worktreeの削除（git worktree remove のラッパー）
```bash
# 通常の削除（ブランチ名またはパスを指定）
twin remove feature-new
twin remove ../feature-new

# 強制削除（エラーを無視）
twin remove feature-new --force

# Git worktree のみ実行（副作用をスキップ）
twin remove feature-new --git-only
```

#### 設定管理
```bash
# デフォルト設定をTOML形式で出力
twin config default

# 現在の設定を表示
twin config --show

# 設定値をセット（未実装）
twin config --set key=value

# 設定値を取得（未実装）
twin config --get key
```

### 未実装機能

以下の機能は現在未実装です：

- `twin config --set/--get` - 設定値の取得・設定
- テンプレート処理

## 設定ファイル

Twin は `.twin.toml` という設定ファイルを使用して副作用を定義します。

### 設定ファイルの例

```toml
# .twin.toml - Twin設定ファイルの例

# Worktreeのベースディレクトリ（省略時: ../ブランチ名）
# worktree_base = "../workspaces"

# ファイルマッピング設定
# Worktree作成時に自動的にシンボリックリンクやコピーを作成します
[[files]]
path = ".env.template"          # ソースファイルのパス
mapping_type = "copy"           # "symlink" または "copy"
description = "環境変数設定"     # 説明（省略可）
skip_if_exists = true           # 既存ファイルをスキップ（省略可）

[[files]]
path = ".claude/config.json"
mapping_type = "symlink"
description = "Claude設定ファイル"

# フック設定（worktree作成・削除時に実行するコマンド）
[hooks]
pre_create = [
  { command = "echo", args = ["Creating: {{branch}}"] }
]
post_create = [
  { command = "npm", args = ["install"], continue_on_error = true }
]
pre_remove = []
post_remove = []
```

### 設定項目の詳細

#### ファイルマッピング (`[[files]]`)

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `path` | string | ✓ | ファイルパス（メインリポジトリとworktreeで同じパス） |
| `mapping_type` | string | - | "symlink" または "copy"（デフォルト: "symlink"） |
| `skip_if_exists` | bool | - | 既存ファイルをスキップ（デフォルト: false） |
| `description` | string | - | マッピングの説明 |

#### フックコマンド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `command` | string | ✓ | 実行するコマンド |
| `args` | array | - | コマンド引数（`{{branch}}`、`{{worktree_path}}`などの変数を使用可） |
| `continue_on_error` | bool | - | エラー時も続行（デフォルト: false） |
| `timeout` | u64 | - | タイムアウト秒数（デフォルト: 60） |

## トラブルシューティング

### Windows でシンボリックリンクが作成できない

**問題**: 「シンボリックリンクの作成に失敗しました」というエラーが表示される

**解決方法**:

1. **開発者モードを有効にする**（推奨）
   - 設定 → 更新とセキュリティ → 開発者向け
   - 「開発者モード」を有効にする

2. **管理者権限で実行する**
   - コマンドプロンプトを管理者として実行
   - または PowerShell を管理者として実行

3. **ファイルコピーモードを使用する**
   - 設定ファイルで `mapping_type = "copy"` を指定

### Worktreeが作成できない

**問題**: 「Worktreeの作成に失敗しました」というエラー

**考えられる原因と解決方法**:

1. **Gitリポジトリではない**
   ```bash
   git init
   ```

2. **同名のブランチが既に存在する**
   ```bash
   # ブランチを確認
   git branch -a
   # 別の名前を指定
   twin add ../agent-001 -b feature/another-name
   # または強制的にブランチを再作成
   twin add ../agent-001 -B feature/new-name
   ```

3. **Worktreeが既に存在する**
   ```bash
   # Worktreeを確認
   twin list
   # または
   git worktree list
   # 既存のWorktreeを削除
   twin remove ../agent-001
   # または
   git worktree remove ../agent-001
   ```

### 設定ファイルが読み込まれない

**問題**: カスタム設定が適用されない

**確認事項**:

1. **設定ファイルの場所**
   - プロジェクトルート: `./.twin.toml`

2. **設定ファイルの形式**
   ```bash
   # デフォルト設定を確認
   twin config default
   ```

3. **TOML構文エラー**
   - 引用符の閉じ忘れ
   - 配列の記法ミス
   - インデントエラー

### デバッグ情報の出力

環境変数を設定してデバッグ情報を表示：

```bash
# 詳細ログを表示
export RUST_LOG=debug
twin add ../agent-001 -b feature/test

# Gitのみ実行（副作用をスキップ）
twin add ../agent-001 --git-only
```

## 開発状況

### 実装済み機能

- ✅ Git worktree コマンドのラッパー（add, list, remove）
- ✅ シンボリックリンク作成・削除（Unix/Windows対応）
- ✅ フック実行（pre/post create/remove）
- ✅ 設定ファイル読み込み（.twin.toml）
- ✅ 複数の出力形式（table, json, simple）
- ✅ エラーハンドリング
- ✅ --git-only オプション（副作用をスキップ）

### 未実装機能

- ⏳ テンプレート処理
- ⏳ 設定値の取得・設定（config --set/--get）

## ライセンス

MIT License

## コントリビューション

プルリクエストを歓迎します。大きな変更の場合は、まずissueを開いて変更内容について議論してください。

## サポート

問題が発生した場合は、GitHubのissuesページで報告してください。
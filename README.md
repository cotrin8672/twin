# Twin - Git Worktree Manager

Twin は Git ワークツリーとシンボリックリンクを使用して、複数のエージェント環境を効率的に管理するためのCLIツールです。

## 概要

各AIエージェントや開発環境ごとに独立したワークツリーを作成し、シンボリックリンクを使用して設定ファイルを共有することで、複数の開発環境を簡単に切り替えることができます。

## 主な機能

- **ワークツリー管理**: Git worktreeを使用した独立した作業環境の作成
- **シンボリックリンク**: 設定ファイルの共有と環境間での一貫性維持
- **クロスプラットフォーム**: Windows/macOS/Linuxに対応
- **フック機能**: 環境作成・削除時のカスタムスクリプト実行
- **柔軟な設定**: TOML形式の設定ファイルによるカスタマイズ

## インストール

```bash
# Rustがインストールされている場合
cargo install --path .

# または、リリースページからバイナリをダウンロード（未実装）
```

## 使用方法

### 基本コマンド

#### 環境の作成
```bash
# 基本的な使用方法
twin create agent-001

# ブランチ名を指定
twin create agent-001 --branch feature/new-feature

# 設定ファイルを指定
twin create agent-001 --config ./custom-config.toml

# パスのみ出力（スクリプト用）
twin create agent-001 --print-path

# cdコマンド形式で出力
twin create agent-001 --cd-command
```

#### 環境の一覧表示
```bash
# デフォルト（テーブル形式）
twin list

# JSON形式で出力
twin list --format json

# シンプルな形式
twin list --format simple
```

#### 環境の削除
```bash
# 通常の削除
twin remove agent-001

# 強制削除（エラーを無視）
twin remove agent-001 --force
```

#### 設定管理
```bash
# 現在の設定を表示
twin config show

# 設定例を表示
twin config example

# 設定ファイルを初期化（未実装）
twin config init

# グローバル設定を編集（未実装）
twin config edit --global
```

### 未実装機能

以下の機能は現在未実装です：

- `twin switch` - 環境の切り替え
- `twin init` - プロジェクトの初期化
- `twin tui` - インタラクティブUI
- `twin config init/edit` - 設定ファイルの初期化と編集
- フック機能の実際の実行

## 設定ファイル

Twin は `twin.toml` という設定ファイルを使用します。プロジェクトルートまたはホームディレクトリに配置できます。

### 設定ファイルの例

```toml
# twin.toml - Twin設定ファイルの例

# ワークツリーのベースディレクトリ（デフォルト: "./worktrees"）
worktree_base = "./worktrees"

# ブランチ名のプレフィックス（デフォルト: "agent/"）
branch_prefix = "agent/"

# ファイルマッピング設定
[[files]]
# ソースファイル（テンプレートまたは共有ファイル）
source = ".claude/config.template.json"
# ターゲットパス（ワークツリー内の配置先）
target = ".claude/config.json"
# マッピングタイプ
# - "symlink": シンボリックリンク（デフォルト）
# - "copy": ファイルコピー
# - "template": テンプレート処理（未実装）
mapping_type = "symlink"
# ファイルが既に存在する場合はスキップ
skip_if_exists = true
# このマッピングの説明
description = "Claude設定ファイル"

[[files]]
source = ".env.template"
target = ".env"
mapping_type = "copy"
skip_if_exists = true
description = "環境変数設定"

[[files]]
source = "shared/hooks"
target = ".git/hooks"
mapping_type = "symlink"
skip_if_exists = false
description = "Gitフック"

# フック設定
[hooks]
# 環境作成前に実行
pre_create = [
    { command = "echo 'Creating environment: {name}'", continue_on_error = false },
    { command = "npm install", continue_on_error = true, timeout = 300 }
]

# 環境作成後に実行
post_create = [
    { command = "echo 'Environment {name} created successfully'", continue_on_error = false },
    { command = "code ./worktrees/{name}", continue_on_error = true }
]

# 環境削除前に実行
pre_remove = [
    { command = "echo 'Removing environment: {name}'", continue_on_error = false }
]

# 環境削除後に実行
post_remove = [
    { command = "echo 'Environment {name} removed'", continue_on_error = false }
]
```

### 設定項目の詳細

#### ファイルマッピング (`[[files]]`)

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `source` | string | ✓ | ソースファイルのパス |
| `target` | string | ✓ | ターゲットファイルのパス（ワークツリー内） |
| `mapping_type` | string | - | "symlink", "copy", "template"（デフォルト: "symlink"） |
| `skip_if_exists` | bool | - | 既存ファイルをスキップ（デフォルト: false） |
| `description` | string | - | マッピングの説明 |

#### フックコマンド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `command` | string | ✓ | 実行するコマンド（`{name}`はエージェント名に置換） |
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

### ワークツリーが作成できない

**問題**: 「ワークツリーの作成に失敗しました」というエラー

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
   twin create agent-001 --branch feature/another-name
   ```

3. **ワークツリーが既に存在する**
   ```bash
   # ワークツリーを確認
   git worktree list
   # 既存のワークツリーを削除
   git worktree remove worktrees/agent-001
   ```

### 設定ファイルが読み込まれない

**問題**: カスタム設定が適用されない

**確認事項**:

1. **設定ファイルの場所**
   - プロジェクトルート: `./twin.toml`
   - ホームディレクトリ: `~/.config/twin/config.toml`（未実装）

2. **設定ファイルの形式**
   ```bash
   # 設定例を確認
   twin config example
   ```

3. **TOML構文エラー**
   - 引用符の閉じ忘れ
   - 配列の記法ミス
   - インデントエラー

### デバッグ情報の出力

環境変数を設定してデバッグ情報を表示：

```bash
# 詳細ログを表示
export TWIN_DEBUG=1
twin create agent-001

# 実行コマンドを表示（dry-run）
export TWIN_DRY_RUN=1
twin create agent-001
```

## 開発状況

### 実装済み機能

- ✅ 基本的なワークツリー管理（create, list, remove）
- ✅ シンボリックリンク作成（Unix/Windows対応）
- ✅ 設定ファイル読み込み
- ✅ 複数の出力形式（table, json, simple）
- ✅ エラーハンドリング

### 未実装機能

- ⏳ 環境切り替え（switch）
- ⏳ プロジェクト初期化（init）
- ⏳ インタラクティブUI（TUI）
- ⏳ フック実行
- ⏳ テンプレート処理
- ⏳ グローバル設定
- ⏳ 設定マージ機能

## ライセンス

MIT License

## コントリビューション

プルリクエストを歓迎します。大きな変更の場合は、まずissueを開いて変更内容について議論してください。

## サポート

問題が発生した場合は、GitHubのissuesページで報告してください。
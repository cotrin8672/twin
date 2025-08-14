# Twin トラブルシューティングガイド

このドキュメントでは、Twin使用時によく発生する問題とその解決方法を説明します。

## 目次

1. [インストール関連](#インストール関連)
2. [Windows固有の問題](#windows固有の問題)
3. [Git/Worktree関連](#gitworktree関連)
4. [シンボリックリンク関連](#シンボリックリンク関連)
5. [設定ファイル関連](#設定ファイル関連)
6. [フック実行関連](#フック実行関連)
7. [デバッグ方法](#デバッグ方法)

## インストール関連

### cargo install が失敗する

**症状**: `cargo install --path .` でエラーが発生

**解決方法**:

1. Rustのバージョンを確認
```bash
rustc --version
# 1.70.0以上が必要
```

2. 依存関係をアップデート
```bash
cargo update
cargo build --release
```

3. クリーンビルド
```bash
cargo clean
cargo build --release
```

## Windows固有の問題

### シンボリックリンクが作成できない

**症状**: 
```
Error: Failed to create symlink: アクセスが拒否されました。 (os error 5)
```

**原因**: Windowsではシンボリックリンク作成に特別な権限が必要

**解決方法**:

#### 方法1: 開発者モードを有効にする（推奨）

1. Windows設定を開く（Win + I）
2. 「更新とセキュリティ」→「開発者向け」
3. 「開発者モード」をオンにする
4. PCを再起動

#### 方法2: 管理者権限で実行

```powershell
# PowerShellを管理者として起動
# スタートメニューで「PowerShell」を右クリック→「管理者として実行」

# Twinコマンドを実行
twin create agent-001
```

#### 方法3: ローカルセキュリティポリシーを変更

1. `secpol.msc` を実行（Win + R）
2. ローカルポリシー → ユーザー権利の割り当て
3. 「シンボリックリンクの作成」を開く
4. 現在のユーザーを追加
5. ログアウト＆ログイン

#### 方法4: ファイルコピーモードを使用

設定ファイルで `mapping_type = "copy"` を指定:

```toml
[[files]]
source = ".env.template"
target = ".env"
mapping_type = "copy"  # シンボリックリンクの代わりにコピー
```

### パスに日本語が含まれる場合のエラー

**症状**: 日本語を含むパスでエラーが発生

**解決方法**:

1. 英語パスに移動
```bash
cd C:\projects\twin
twin create agent-001
```

2. 環境変数でエンコーディングを設定
```powershell
$env:LANG = "ja_JP.UTF-8"
```

## Git/Worktree関連

### "not a git repository" エラー

**症状**: 
```
Error: Git error: not a git repository
```

**解決方法**:

```bash
# Gitリポジトリを初期化
git init

# または既存のリポジトリをクローン
git clone <repository-url>
cd <repository-name>
```

### ブランチが既に存在する

**症状**:
```
Error: Git error: branch 'agent/agent-001' already exists
```

**解決方法**:

1. 既存のブランチを確認
```bash
git branch -a | grep agent-001
```

2. 別の名前を指定
```bash
twin create agent-001 --branch feature/agent-001-v2
```

3. または既存のブランチを削除
```bash
git branch -D agent/agent-001
twin create agent-001
```

### ワークツリーが既に存在する

**症状**:
```
Error: Git error: worktree already exists
```

**解決方法**:

1. 既存のワークツリーを確認
```bash
git worktree list
```

2. 不要なワークツリーを削除
```bash
git worktree remove worktrees/agent-001
# または強制削除
git worktree remove --force worktrees/agent-001
```

3. プルーニング（クリーンアップ）
```bash
git worktree prune
```

### メインブランチにコミットがない

**症状**:
```
Error: Git error: no commits yet on 'main'
```

**解決方法**:

```bash
# 初期コミットを作成
echo "# My Project" > README.md
git add README.md
git commit -m "Initial commit"
```

## シンボリックリンク関連

### リンク先が見つからない

**症状**:
```
Error: Symlink error: source file not found
```

**解決方法**:

1. ソースファイルの存在を確認
```bash
ls -la .claude/config.template.json
```

2. 設定ファイルのパスを修正
```toml
[[files]]
source = "existing-file.json"  # 存在するファイルを指定
target = ".config/app.json"
```

### シンボリックリンクが壊れている

**症状**: リンクは存在するが、ファイルが開けない

**診断方法**:

```bash
# Unix/macOS
ls -la worktrees/agent-001/.config/

# Windows PowerShell
Get-ChildItem worktrees\agent-001\.config\ -Force
```

**解決方法**:

```bash
# 環境を再作成
twin remove agent-001 --force
twin create agent-001
```

## 設定ファイル関連

### 設定ファイルが読み込まれない

**症状**: カスタム設定が適用されない

**診断**:

1. 設定ファイルの場所を確認
```bash
# プロジェクトルートに twin.toml があるか確認
ls -la twin.toml
```

2. TOML構文を検証
```bash
# オンラインTOMLバリデータを使用
# または cargo-toml-validate をインストール
cargo install cargo-toml-validate
cargo toml-validate twin.toml
```

### TOML構文エラー

**よくあるエラー**:

1. 文字列の引用符忘れ
```toml
# ❌ 間違い
source = path/to/file

# ✅ 正しい
source = "path/to/file"
```

2. 配列の記法ミス
```toml
# ❌ 間違い
pre_create = { command = "echo test" }

# ✅ 正しい
pre_create = [
    { command = "echo test" }
]
```

3. 重複したキー
```toml
# ❌ 間違い
[hooks]
pre_create = [...]
[hooks]  # 重複
post_create = [...]

# ✅ 正しい
[hooks]
pre_create = [...]
post_create = [...]
```

## フック実行関連

### フックが実行されない（未実装）

**現状**: フック機能は現在未実装です

**回避策**:

手動でスクリプトを実行:
```bash
# 環境作成後
twin create agent-001
./scripts/post-create.sh agent-001

# 環境削除前
./scripts/pre-remove.sh agent-001
twin remove agent-001
```

### フックコマンドのエラー

**症状**: フックコマンドが失敗する

**デバッグ方法**:

1. コマンドを手動で実行
```bash
# {name} を実際の値に置換して実行
echo 'Creating environment: agent-001'
```

2. continue_on_error を使用
```toml
{ command = "risky-command", continue_on_error = true }
```

## デバッグ方法

### 詳細ログの有効化

```bash
# Bashの場合
export TWIN_DEBUG=1
export TWIN_VERBOSE=1
twin create agent-001

# PowerShellの場合
$env:TWIN_DEBUG = "1"
$env:TWIN_VERBOSE = "1"
twin create agent-001

# Rustのログも有効化
export RUST_LOG=twin=debug
twin create agent-001
```

### Dry-runモード（未実装）

将来的に実装予定:
```bash
export TWIN_DRY_RUN=1
twin create agent-001
```

### よく使うデバッグコマンド

```bash
# Git状態の確認
git status
git worktree list
git branch -a

# ファイルシステムの確認
ls -la worktrees/
tree worktrees/agent-001/

# Windows: シンボリックリンクの確認
dir /A:L worktrees\agent-001\

# プロセスの確認（フック用）
ps aux | grep twin
tasklist | findstr twin
```

## 一般的な解決手順

問題が解決しない場合の一般的なアプローチ:

1. **クリーンアップ**
```bash
# ワークツリーをクリーンアップ
git worktree prune
# Twinの環境を削除
twin remove agent-001 --force
```

2. **設定をリセット**
```bash
# 設定ファイルをバックアップ
cp twin.toml twin.toml.backup
# デフォルト設定で実行
rm twin.toml
twin create test-agent
```

3. **最小構成でテスト**
```bash
# 最小限の設定ファイル
echo 'worktree_base = "./worktrees"' > twin.toml
twin create test-agent
```

4. **問題の報告**

GitHubでissueを作成する際に含める情報:
- OS種類とバージョン
- Rustバージョン (`rustc --version`)
- Gitバージョン (`git --version`)
- エラーメッセージ全文
- 実行したコマンド
- 設定ファイルの内容（機密情報は除く）

## サポート

解決しない場合は、GitHubのissuesで報告してください:
https://github.com/yourusername/twin/issues
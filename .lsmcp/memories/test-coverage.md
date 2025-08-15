---
created: 2025-08-15T17:04:57.763Z
updated: 2025-08-15T17:04:57.763Z
---

# Twin テストカバレッジ詳細

## テスト実行環境
- **Linuxコンテナ環境のみ**: `rust:1.75-slim` Dockerイメージを使用
- **ローカル環境汚染なし**: すべてのGit操作はコンテナ内で実行
- **一意のテストID**: 各テストインスタンスに UUID を付与してパス衝突を防止

## E2Eテスト (`tests/e2e_container_test.rs`)

### 1. test_full_workflow
**目的**: 基本的なワークフロー全体の動作確認
**テスト内容**:
- `twin add` でworktree作成（新規ブランチ付き）
- `twin list` で作成したworktreeが表示されることを確認
- `twin remove --force` でworktree削除
- 削除後、listに表示されないことを確認

### 2. test_worktree_options
**目的**: Git worktreeの各種オプションが正しく動作することを確認
**テスト内容**:
- `--detach HEAD`: デタッチモードでworktree作成
- `--no-checkout`: ファイルをチェックアウトせずにworktree作成
- `--quiet`: 静かなモードで最小限の出力

### 3. test_git_only_mode
**目的**: `--git-only`モードで追加機能がスキップされることを確認
**テスト内容**:
- `.twin.toml`設定ファイルを作成（シンボリックリンクとフック定義）
- `--git-only`フラグ付きでworktree作成
- フックが実行されないことを確認
- シンボリックリンクが作成されないことを確認

### 4. test_error_handling
**目的**: エラー処理とメッセージの透過性を確認
**テスト内容**:
- 存在しないブランチを指定 → Gitのエラーメッセージが表示される
- 無効なブランチ名（`..invalid..`）→ エラーになる
- 既存のディレクトリにworktree作成 → "already exists"エラー

### 5. test_path_handling
**目的**: パス処理の正確性を確認
**テスト内容**:
- 相対パス（`../relative-xxx`）でworktree作成
- 絶対パス（`/tmp/absolute-xxx`）でworktree作成
- Windowsスタイルのパス処理（現在はLinuxコンテナのみなので未実装）

### 6. test_compatibility_with_git_worktree
**目的**: 既存のgit worktreeコマンドとの互換性を確認
**テスト内容**:
- `git worktree add`で直接worktree作成
- `twin list`で表示されることを確認（相互運用性）
- `twin remove`で削除できることを確認
- `git worktree list`で削除されたことを確認

## 結合テスト (`tests/integration_container_test.rs`)

### 1. test_git_integration
**目的**: Gitブランチ操作との結合動作を確認
**テスト内容**:
- `-b`オプションで新規ブランチ作成を伴うworktree追加
- `git branch -a`でブランチが作成されたことを確認
- 既存ブランチ名での作成がエラーになることを確認
- `-B`オプションで強制的にブランチを再作成

### 2. test_symlink_integration
**目的**: シンボリックリンク機能の動作確認
**テスト内容**:
- `.twin.toml`でシンボリックリンク設定を定義
- worktree作成時にシンボリックリンクが作成される
- `--git-only`モードではシンボリックリンクがスキップされる
- ファイルとディレクトリの両方のリンクをサポート

### 3. test_hook_integration
**目的**: フック実行機能の動作確認
**テスト内容**:
- `post_create`フックの定義と実行
- フック実行結果の確認（マーカーファイル作成）
- `--git-only`モードではフックがスキップされる

### 4. test_complex_workflow
**目的**: 複数worktreeの管理と操作を確認
**テスト内容**:
- 複数のworktreeを連続作成（3つ）
- `twin list`ですべて表示されることを確認
- worktree内でファイル変更とコミット
- 変更があるworktreeの削除には`--force`が必要
- すべてのworktreeを削除して空になることを確認

## 既存の基本テスト (`tests/git_worktree_wrapper_test.rs`)
**注**: このファイルはローカル実行用で、現在は使用非推奨

### カバーしている項目:
- addコマンドの基本動作
- Git worktreeの各種オプション（-B, --detach, --lock, --no-checkout, --quiet）
- --git-onlyモード
- エラーメッセージの透過性
- 既存worktreeとの互換性
- 出力フォーマットの一致性

## テストヘルパー (`tests/common/mod.rs`)

### TestRepo構造体
- **環境**: Linuxコンテナのみサポート
- **test_id**: 各テストインスタンスの一意識別子
- **worktree_path()**: 一意のworktreeパスを生成
- **exec()**: コンテナ内でコマンド実行
- **run_twin()**: twinバイナリをコンテナにコピーして実行

### セットアップ手順:
1. Dockerコンテナ起動（rust:1.75-slim）
2. Git環境のセットアップ
3. テスト用リポジトリの初期化
4. 初期コミット作成
5. twinバイナリのコンテナへのコピー

## カバレッジのギャップ

### 未テストの機能:
- `twin move` コマンド
- `twin prune` コマンド
- `twin repair` コマンド
- 複数の設定ファイルのマージ
- 環境変数による設定のオーバーライド
- ロック機能の詳細な動作

### 将来的に追加すべきテスト:
- パフォーマンステスト（大規模リポジトリ）
- 並行実行時の動作
- 中断されたoperationのリカバリ
- ネットワークドライブ上での動作（Windows環境）
# Implementation Plan

## 薄いGit Worktreeラッパーへのシンプルなリファクタリング

既存のtwinを「git worktree + 副作用」のシンプルなラッパーに変更する。複雑な機能は追加せず、git worktreeの基本操作に副作用（シンボリックリンク、フック）を付加するだけ。

- [x] 1. git worktreeの薄いラッパー実装
  - [x] 1.1 基本コマンドの変更
    - `src/cli.rs`で`create` → `add`、`remove` → `remove`に変更
    - `twin add <path> [<branch>]`でgit worktree addと同じ引数を受け入れる
    - `twin list`でgit worktree listと同じ出力を生成
    - `twin remove <worktree>`でgit worktree removeと同じ動作
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [x] 1.2 git worktreeコマンドの直接実行
    - `src/git.rs`のGitManagerで実際のgit worktreeコマンドを実行
    - 引数やオプションをそのままgit worktreeに渡す
    - エラーメッセージもgit worktreeのものをそのまま表示
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 7.1, 7.2, 7.3_

- [x] 2. 副作用の簡単な実装
  - [x] 2.1 シンボリックリンク副作用
    - `src/symlink.rs`の既存機能を使用
    - worktree作成後にシンボリックリンクを作成
    - worktree削除前にシンボリックリンクを削除
    - エラー時は警告表示して継続
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

  - [x] 2.2 フック副作用
    - `src/hooks.rs`の既存機能を使用
    - worktree作成前後、削除前後にフックを実行
    - エラー時は設定に応じて継続/中断
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 3. 自前レジストリの廃止
  - [x] 3.1 EnvironmentManagerの削除
    - `src/environment.rs`の自前レジストリ機能を完全削除
    - `EnvironmentRegistry`、`AgentEnvironment`等の型を削除
    - `.git/twin-registry.json`の読み書き処理を削除
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [x] 3.2 git worktree listベースの一覧表示
    - `twin list`は単純に`git worktree list`の結果を表示（整形付き）
    - メインリポジトリとワークツリーを区別して表示
    - _Requirements: 1.2, 7.1, 7.2, 7.3_

- [x] 4. 設定の簡素化
  - [x] 4.1 既存設定の保持
    - `src/core/types.rs`の設定構造を保持
    - シンボリックリンク設定（`files`）とフック設定（`hooks`）のみ使用
    - 複雑な副作用設定は追加しない
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5. コマンド実装の簡素化
  - [x] 5.1 `twin add`の実装
    - `src/cli/commands.rs`の`handle_create`を`handle_add`に変更
    - git worktree addを実行
    - 成功後にシンボリックリンクとフックを実行
    - _Requirements: 1.1, 2.1, 2.2, 2.3, 2.4_

  - [x] 5.2 `twin remove`の実装
    - `handle_remove`を修正
    - 削除前にフックを実行
    - git worktree removeを実行
    - _Requirements: 1.3, 2.1, 2.2, 2.3, 2.4_

  - [x] 5.3 `twin list`の実装
    - `handle_list`を修正
    - git worktree listの結果を整形して表示
    - _Requirements: 1.2_

- [x] 6. テストの更新
  - [x] 6.1 既存テストの修正
    - 自前レジストリ関連のテストを削除
    - git worktree互換性のテストを追加
    - _Requirements: 全要件のテストカバレッジ_

## 実装の優先順位

1. **Phase 1**: 自前レジストリの廃止（タスク3）
2. **Phase 2**: git worktreeラッパーの実装（タスク1）
3. **Phase 3**: 副作用の統合（タスク2）
4. **Phase 4**: コマンド実装（タスク5）
5. **Phase 5**: テスト更新（タスク6）

## 既存コードの保持/変更方針

### そのまま使用するもの
- `src/git.rs` - GitManagerの基本機能（git worktreeコマンド実行）
- `src/symlink.rs` - 既存のSymlinkManager（副作用として使用）
- `src/hooks.rs` - 既存のHookExecutor（副作用として使用）
- `src/config.rs` - 設定読み込み機能
- `src/core/error.rs` - エラー型
- `src/cli/output.rs` - 出力フォーマット機能

### 削除するもの
- `src/environment.rs` - 自前レジストリ機能を完全削除
- `src/core/types.rs`の`EnvironmentRegistry`、`AgentEnvironment`等

### 軽微な変更
- `src/cli.rs` - `create` → `add`にコマンド名変更
- `src/cli/commands.rs` - git worktreeを直接実行し、副作用を付加
- `src/core/types.rs` - レジストリ関連型を削除
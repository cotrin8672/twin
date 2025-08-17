# Requirements Document

## Introduction

この機能は、Twinでworktree環境作成後に、CLIコーディングエージェント（Claude Code、Gemini CLIなど）を自動起動し、作成されたworktree環境にセッションをアタッチする機能です。既存のpost_createフック機能を拡張し、エージェント起動とセッション管理を統合することで、開発者がworktree作成後すぐにAI支援開発を開始できるシームレスな体験を提供します。

## Requirements

### Requirement 1

**User Story:** 開発者として、`twin add`でworktree作成後に自動的にコーディングエージェントが起動されることで、手動でエージェントを起動する手間なくすぐにAI支援開発を開始したい

#### Acceptance Criteria

1. WHEN `twin add`コマンドでworktreeが正常に作成された THEN システムは設定されたコーディングエージェントをpost_createフックとして自動起動する SHALL
2. WHEN エージェントが起動される THEN システムは作成されたworktreeディレクトリを作業ディレクトリとしてエージェントを起動する SHALL
3. WHEN エージェントが起動される THEN システムはworktreeのパス、ブランチ名、プロジェクト情報をエージェントに渡す SHALL
4. IF エージェント起動が失敗した THEN システムはエラーメッセージを表示し、worktree作成は成功として継続する SHALL

### Requirement 2

**User Story:** 開発者として、twin.tomlでエージェント設定を管理できることで、プロジェクトごとに適切なエージェントを自動起動したい

#### Acceptance Criteria

1. WHEN twin.tomlにエージェント設定を記述する THEN システムはClaude Code（`claude-dev`）、Gemini CLI（`gemini`）、その他のCLIエージェントをサポートする SHALL
2. WHEN エージェント設定でコマンドと引数を指定する THEN システムは指定されたコマンドラインでエージェントを起動する SHALL
3. WHEN エージェント設定で環境変数を指定する THEN システムは指定された環境変数を設定してエージェントを起動する SHALL
4. IF エージェント設定が無効または不完全な場合 THEN システムはエラーメッセージを表示し、エージェント起動をスキップする SHALL

### Requirement 3

**User Story:** 開発者として、エージェント起動時にworktreeのコンテキスト情報が自動的に渡されることで、エージェントがプロジェクトの状況を理解した状態で開始したい

#### Acceptance Criteria

1. WHEN エージェントが起動される THEN システムはworktreeパス、ブランチ名、エージェント名を環境変数として設定する SHALL
2. WHEN エージェントが起動される THEN システムは作業ディレクトリをworktreeディレクトリに設定する SHALL
3. WHEN エージェントが起動される THEN システムはプロジェクトルートパスを環境変数として設定する SHALL
4. IF エージェントが初期プロンプトをサポートしている THEN システムは設定可能な初期メッセージを送信する SHALL

### Requirement 4

**User Story:** 開発者として、エージェント起動を手動で制御できることで、必要に応じてエージェント機能を有効・無効にしたい

#### Acceptance Criteria

1. WHEN `twin add --no-agent`オプションを指定する THEN システムはエージェント起動をスキップする SHALL
2. WHEN `twin add --agent <agent_name>`オプションを指定する THEN システムは指定されたエージェントのみを起動する SHALL
3. WHEN twin.tomlでエージェント機能を無効化する THEN システムはエージェント起動をスキップする SHALL
4. WHEN 既存のworktreeでエージェントを起動したい THEN システムは`twin agent start <worktree_name>`コマンドでエージェントを起動できる SHALL

### Requirement 5

**User Story:** 開発者として、エージェントセッション管理機能により、実行中のエージェントの状態を把握し制御したい

#### Acceptance Criteria

1. WHEN `twin agent list`コマンドを実行する THEN システムは実行中のエージェントセッション一覧を表示する SHALL
2. WHEN `twin agent stop <worktree_name>`コマンドを実行する THEN システムは指定されたworktreeのエージェントセッションを終了する SHALL
3. WHEN `twin agent attach <worktree_name>`コマンドを実行する THEN システムは既存のエージェントセッションに再接続する SHALL
4. IF エージェントプロセスが予期せず終了した THEN システムはプロセス状態を追跡し、次回のlist実行時に状態を更新する SHALL

### Requirement 6

**User Story:** 開発者として、既存のTwinフック機能と統合されたエージェント機能により、一貫した設定管理と実行体験を得たい

#### Acceptance Criteria

1. WHEN エージェント起動設定を行う THEN システムは既存のpost_createフック設定と同じ形式で設定できる SHALL
2. WHEN エージェント起動が実行される THEN システムは既存のフック実行システム（HookExecutor）を使用する SHALL
3. WHEN エージェント起動でエラーが発生する THEN システムは既存のフックエラーハンドリング（continue_on_error）を適用する SHALL
4. WHEN `--git-only`オプションを指定する THEN システムはエージェント起動を含む全ての副作用をスキップする SHALL
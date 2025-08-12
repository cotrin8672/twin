# Requirements Document

## Introduction

AI開発エージェント（ClaudeCode等）用の軽量な環境分離ツール。Git Worktreeとシンボリックリンクを活用して、各エージェントに独立した作業空間を提供する。従来のペイン分割フローを維持しながら、worktree作成・ディレクトリ移動・環境設定を簡素化し、ccmanagerの不透明性を解決する。クロスプラットフォーム対応（Linux/Windows）で、エージェントの操作が透明に見える環境管理を実現する。

## Requirements

### Requirement 1

**User Story:** 開発者として、エージェント用のworktree環境を素早く作成し、そこに移動したい。そうすることで、「worktree作成 → cd → エージェント起動」の流れを簡単に実行できる。

#### Acceptance Criteria

1. WHEN ユーザーが環境作成コマンドを実行する THEN システムは一意のエージェント名を生成し、対応するworktreeを作成する SHALL
2. WHEN worktree作成時 THEN システムは現在のブランチをベースにした新しいブランチを自動作成し、エージェント名を含むブランチ名を付ける SHALL
3. WHEN worktree作成が完了する THEN システムは作成されたworktreeのパスを出力し、cdコマンドを提示する SHALL
4. WHEN ユーザーが `--cd` オプションを指定する THEN システムは作成後に自動的にそのディレクトリに移動する SHALL

### Requirement 2

**User Story:** 開発者として、Git管理外のファイル（環境変数、リファレンス資料等）に各エージェント環境からアクセスしたい。そうすることで、Container-useと同様の利便性を保ちながら、軽量な環境分離を実現できる。

#### Acceptance Criteria

1. WHEN エージェント環境作成時 THEN システムは設定で指定されたディレクトリ・ファイルへのシンボリックリンクを作成する SHALL
2. WHEN Linuxで実行される THEN システムは `ln -s` コマンドでシンボリックリンクを作成する SHALL
3. WHEN Windowsで実行される THEN システムは `mklink` コマンドでシンボリックリンクを作成する SHALL
4. WHEN リンク作成に失敗する THEN システムはエラーメッセージを表示し、手動でのリンク作成方法を提示する SHALL
5. WHEN エージェント環境削除時 THEN システムは作成したシンボリックリンクも併せて削除する SHALL

### Requirement 3

**User Story:** 開発者として、エージェントの作業を自動的にコミットとして記録したい。そうすることで、Container-useのように作業履歴を失うことなく、各エージェントの進捗を追跡できる。

#### Acceptance Criteria

1. WHEN エージェント環境で変更が発生する THEN システムは定期的に変更を検出し、自動コミットを実行する SHALL
2. WHEN 自動コミット実行時 THEN システムはエージェント名と変更内容を含む意味のあるコミットメッセージを生成する SHALL
3. WHEN ユーザーが手動コミットを実行する THEN システムは自動コミット機能を一時停止し、手動コミット後に再開する SHALL
4. WHEN エージェント環境削除時 THEN システムは未コミットの変更があれば最終コミットを実行してから削除する SHALL

### Requirement 4

**User Story:** 開発者として、エージェントの操作を透明に確認しながら作業したい。そうすることで、ccmanagerのような不透明性を避け、標準的なCLIツールとして任意の環境（ペイン分割含む）で使用できる。

#### Acceptance Criteria

1. WHEN エージェント環境を作成する THEN システムは実行されるGitコマンドを表示する SHALL
2. WHEN シンボリックリンクを作成する THEN システムは作成されるリンクの詳細を表示する SHALL
3. WHEN 環境削除を実行する THEN システムは削除される内容を事前に表示し、確認を求める SHALL
4. WHEN 全ての操作を実行する THEN システムは標準的なCLIツールとして動作し、ペイン分割やターミナル多重化ツールと互換性を保つ SHALL

### Requirement 5

**User Story:** 開発者として、慣れた後はCLIでワンライナーで操作したい。そうすることで、スクリプト化や高速な操作が可能になり、開発フローに組み込める。

#### Acceptance Criteria

1. WHEN ユーザーがCLIコマンドを実行する THEN システムは全ての機能をコマンドライン引数で実行可能にする SHALL
2. WHEN CLIでエージェント環境を作成する THEN システムは `create <name>` のような簡潔なコマンドで実行できる SHALL
3. WHEN CLIで環境一覧を表示する THEN システムは `list` コマンドで表形式の見やすい出力を提供する SHALL
4. WHEN CLIで環境切り替えを実行する THEN システムは `switch <name>` コマンドでディレクトリ変更コマンドを出力する SHALL

### Requirement 6

**User Story:** 開発者として、プロジェクトごとの設定をカスタマイズしたい。そうすることで、プロジェクトの特性に合わせてリンク対象や自動コミット設定を最適化できる。

#### Acceptance Criteria

1. WHEN ユーザーが設定ファイルを作成する THEN システムはworktreeの作成場所、リンク対象、自動コミット間隔を設定できる SHALL
2. WHEN 設定ファイルが存在しない THEN システムはデフォルト設定を使用する SHALL
3. WHEN 無効な設定が指定される THEN システムは適切なエラーメッセージと設定例を表示する SHALL
4. WHEN プロジェクトルートに設定ファイルが存在する THEN システムはそれをグローバル設定より優先する SHALL
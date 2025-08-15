# Requirements Document

## Introduction

twinコマンドをGit Worktreeの純粋なラッパーとして再設計し、git worktreeの全ての利便性を保持しながら、副作用システムによる拡張機能を提供するツール。従来の自前レジストリ実装を廃止し、git worktreeの標準的な動作を最大限活用する。全ての追加機能（シンボリックリンク、フック実行、環境設定等）は「副作用」として定義し、将来的な機能拡張も同じ仕組みで実現できる拡張性の高いアーキテクチャを採用する。開発者はgit worktreeの標準的な使い方を学ぶだけで、twinの恩恵を受けられる。

## Requirements

### Requirement 1

**User Story:** 開発者として、git worktreeの標準的な操作をそのまま使いながら、副作用による拡張機能を自動実行したい。そうすることで、git worktreeの学習コストゼロで、追加の利便性を得られる。

#### Acceptance Criteria

1. WHEN ユーザーが `twin add <path> [<branch>]` を実行する THEN システムは `git worktree add` と完全に同じ引数・オプションを受け入れ、git操作完了後に設定された副作用を実行する SHALL
2. WHEN ユーザーが `twin list` を実行する THEN システムは `git worktree list` と同じ出力形式を維持し、オプションで副作用の状態情報を追加表示できる SHALL
3. WHEN ユーザーが `twin remove <worktree>` を実行する THEN システムは `git worktree remove` と同じ動作をし、削除前に関連副作用のクリーンアップを実行する SHALL
4. WHEN ユーザーが `twin prune` を実行する THEN システムは `git worktree prune` を実行し、孤立した副作用リソースも自動検出・クリーンアップする SHALL

### Requirement 2

**User Story:** 開発者として、worktree操作の前後で自動的に副作用を実行したい。そうすることで、手動での環境設定作業を省略し、一貫した開発環境を維持できる。

#### Acceptance Criteria

1. WHEN worktree作成時 THEN システムは設定されたpre-addフックを実行してから、git worktree addを実行する SHALL
2. WHEN worktree作成完了後 THEN システムは設定されたシンボリックリンクを作成し、post-addフックを実行する SHALL
3. WHEN worktree削除時 THEN システムは設定されたpre-removeフックを実行してから、シンボリックリンクを削除し、git worktree removeを実行する SHALL
4. WHEN worktree削除完了後 THEN システムは設定されたpost-removeフックを実行し、関連リソースをクリーンアップする SHALL

### Requirement 3

**User Story:** 開発者として、worktreeの状態と副作用の状態を統合的に確認したい。そうすることで、環境の整合性を保ち、問題の早期発見ができる。

#### Acceptance Criteria

1. WHEN ユーザーが `twin status` を実行する THEN システムは各worktreeの状態（ブランチ、コミット、変更状況）と副作用の状態（シンボリックリンクの有効性等）を表示する SHALL
2. WHEN ユーザーが `twin check` を実行する THEN システムは全worktreeの整合性をチェックし、問題があれば修復方法を提示する SHALL
3. WHEN ユーザーが `twin repair <worktree>` を実行する THEN システムは指定されたworktreeの副作用を再構築し、整合性を回復する SHALL
4. WHEN 副作用に問題がある THEN システムは警告を表示し、自動修復または手動修復の選択肢を提供する SHALL

### Requirement 4

**User Story:** 開発者として、Git Worktreeの標準的な使い方を維持しながら、twinの拡張機能を利用したい。そうすることで、学習コストを最小化し、既存のワークフローに自然に統合できる。

#### Acceptance Criteria

1. WHEN ユーザーがgit worktreeコマンドを直接実行する THEN システムはそれを検出し、必要に応じて副作用の同期を提案する SHALL
2. WHEN ユーザーが `twin sync` を実行する THEN システムは既存のworktreeに対して不足している副作用を追加適用する SHALL
3. WHEN ユーザーが `--git-only` オプションを指定する THEN システムは副作用を実行せず、純粋なgit worktree操作のみを実行する SHALL
4. WHEN ユーザーが `--dry-run` オプションを指定する THEN システムは実行予定の操作（git操作と副作用）を表示し、実際の実行は行わない SHALL

### Requirement 5

**User Story:** 開発者として、worktree操作に関連する副作用を柔軟に設定したい。そうすることで、プロジェクトの特性や個人の好みに合わせて動作をカスタマイズできる。

#### Acceptance Criteria

1. WHEN ユーザーが設定ファイルを作成する THEN システムはworktree操作ごとの副作用（シンボリックリンク、フック、環境変数等）を定義できる SHALL
2. WHEN 設定ファイルでシンボリックリンクを定義する THEN システムはソースパスとターゲットパスのペアを複数指定でき、相対パス・絶対パスの両方をサポートする SHALL
3. WHEN 設定ファイルでフックを定義する THEN システムはpre-add/post-add/pre-remove/post-removeの各タイミングでシェルコマンドを実行できる SHALL
4. WHEN 設定ファイルで条件分岐を定義する THEN システムはworktreeのパスやブランチ名に基づいて異なる副作用を適用できる SHALL

### Requirement 6

**User Story:** 開発者として、worktree間での作業を効率的に切り替えたい。そうすることで、複数の作業を並行して進める際の移動コストを削減できる。

#### Acceptance Criteria

1. WHEN ユーザーが `twin switch <worktree>` を実行する THEN システムは指定されたworktreeディレクトリへの移動コマンドを出力する SHALL
2. WHEN ユーザーが `twin cd <worktree>` を実行する THEN システムはシェル関数/エイリアス経由で実際にディレクトリを変更する SHALL
3. WHEN ユーザーが `twin recent` を実行する THEN システムは最近使用したworktreeの履歴を表示し、番号選択で切り替えできる SHALL
4. WHEN ユーザーが環境変数 `TWIN_AUTO_CD=1` を設定する THEN システムはworktree作成後に自動的にそのディレクトリに移動する SHALL
### Requ
irement 7

**User Story:** 開発者として、git worktreeの標準機能を一切損なうことなく、twinの副作用システムを利用したい。そうすることで、既存のgit worktreeワークフローを変更せずに、段階的にtwinを導入できる。

#### Acceptance Criteria

1. WHEN ユーザーがgit worktreeで作成したworktreeが存在する THEN twinはそれを自動認識し、副作用の適用を提案する SHALL
2. WHEN twinが管理していないworktreeに対して副作用を適用する THEN システムは既存の状態を保持し、追加のみを行う SHALL
3. WHEN git worktreeの標準コマンドが直接実行される THEN twinはそれを妨げず、必要に応じて副作用の同期を後から実行できる SHALL
4. WHEN twinを削除する THEN git worktreeの標準機能は完全に保持され、副作用のみがクリーンアップされる SHALL

### Requirement 8

**User Story:** 開発者として、副作用システムを通じて将来的な機能拡張を簡単に追加したい。そうすることで、プラグインのような形で新しい機能を組み込める。

#### Acceptance Criteria

1. WHEN 新しい副作用タイプを定義する THEN システムは既存のworktree操作に影響を与えずに、新機能を統合できる SHALL
2. WHEN 副作用の実行順序を制御する THEN システムは依存関係や優先度に基づいて、適切な順序で副作用を実行する SHALL
3. WHEN 副作用が失敗する THEN システムはgit worktree操作の成功/失敗とは独立して、副作用のエラーハンドリングを行う SHALL
4. WHEN 副作用を無効化する THEN システムは設定により個別の副作用を選択的に無効化でき、git worktree操作のみを実行できる SHALL
# Twin Project Overview

## プロジェクトの目的
Twinは、Gitワークツリーとシンボリックリンクを使用した開発環境管理ツールです。複数の開発環境（エージェント環境）を作成・管理し、プロジェクトファイルのシンボリックリンクを通じて異なるブランチや設定で同時に作業できるようにします。

## 主な機能
- **環境作成**: 新しいエージェント環境を作成（`twin create`）
- **環境一覧**: 既存の環境をリスト表示（`twin list`）
- **環境削除**: 不要な環境を削除（`twin remove`）
- **設定管理**: グローバル/ローカル設定の管理（`twin config`）
- **TUI**: ターミナルUIインターフェース（未実装）

## 技術スタック
- **言語**: Rust (Edition 2024)
- **非同期ランタイム**: Tokio
- **CLI**: Clap v4
- **設定**: TOML/JSON (serde)
- **Git操作**: git2
- **TUI**: Ratatui + Crossterm
- **ログ**: tracing/tracing-subscriber
- **エラーハンドリング**: anyhow + thiserror

## アーキテクチャ
- `src/main.rs`: エントリーポイント
- `src/cli/`: CLIコマンドとパーサー
- `src/core/`: コアタイプとエラー定義
- `src/platform/`: プラットフォーム固有の実装
- `src/utils/`: ユーティリティ関数
- `tests/`: 統合テストとE2Eテスト
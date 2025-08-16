# Twin - Git ワークツリーマネージャー

[[English|Home]] | **日本語**

**Git ワークツリー操作を簡素化し、その副作用を管理するラッパーツール**

## プロジェクト概要

Twin は、Git のワークツリー機能をインテリジェントにラップする Rust 製のコマンドラインツールです。その中核となる思想は、Git ワークツリーを使いやすくすると同時に、複数の作業ディレクトリの作成と保守に伴う副作用（エフェクト）を自動的に管理することです。

新しいワークツリーを作成する際、Twin は Git 操作を処理するだけでなく、必要なエフェクトも管理します：
- **エフェクト管理**: ワークツリー操作のエフェクトとして、シンボリックリンク、ファイルマッピング、フックを自動処理
- **一貫性**: 各ワークツリーが適切な設定とセットアップを持つことを保証
- **シンプルさ**: 複雑な Git ワークツリー操作にシンプルなインターフェースを提供

### 主な機能
- 🌲 **Git ワークツリーラッパー** - Git ワークツリー操作の簡素化されたインターフェース
- 🎯 **エフェクト管理** - ワークツリー関連の副作用の自動処理
- 🔗 **シンボリックリンクエフェクト** - ワークツリー作成のエフェクトとしてシンボリックリンクを作成
- 🪝 **フックエフェクト** - ワークツリーライフサイクルのエフェクトとしてセットアップコマンドを実行
- 🖥️ **クロスプラットフォーム** - Windows、macOS、Linux で一貫した動作
- ⚙️ **宣言的設定** - TOML 設定でエフェクトを定義

## クイックナビゲーション

### はじめに
- [開発環境のセットアップ](JA-Getting-Started-Development-Setup)
- [クイックスタートガイド](JA-Getting-Started-Quick-Start)
- [アーキテクチャ概要](JA-Getting-Started-Architecture-Overview)
- [コア概念](JA-Getting-Started-Core-Concepts)

### 技術ドキュメント
- [技術スタック](JA-Architecture-Technology-Stack)
- [システム設計](JA-Architecture-System-Design)
- [モジュール構造](JA-Architecture-Module-Structure)
- [データフロー](JA-Architecture-Data-Flow)

### 開発ガイド
- [ローカル開発](JA-Development-Guides-Local-Development)
- [デバッグ](JA-Development-Guides-Debugging)
- [コードスタイル](JA-Development-Guides-Code-Styles)

### テスト
- [テスト戦略](JA-Testing-Testing-Strategy)
- [ユニットテスト](JA-Testing-Unit-Testing)
- [統合テスト](JA-Testing-Integration-Testing)
- [E2E テスト](JA-Testing-E2E-Testing)

### デプロイメント
- [ビルドプロセス](JA-Deployment-Build-Process)
- [設定](JA-Deployment-Configuration)

## 開発者リソース

- **リポジトリ**: [GitHub](https://github.com/your-org/twin)
- **課題トラッカー**: [GitHub Issues](https://github.com/your-org/twin/issues)
- **パッケージ**: [crates.io](https://crates.io/crates/twin) (準備中)
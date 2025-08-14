# コードスタイルと規約

## Rustコーディング規約
- **Edition**: Rust 2024
- **エラーハンドリング**: `anyhow::Result`を使用、カスタムエラーは`thiserror`
- **非同期**: `#[tokio::main]`と`async/await`
- **ログ**: `tracing`マクロを使用（`info!`, `debug!`, `error!`など）

## 命名規則
- **構造体/列挙型**: PascalCase（例: `AgentEnvironment`, `EnvironmentStatus`）
- **関数/メソッド**: snake_case（例: `handle_create`, `get_config`）
- **定数**: UPPER_SNAKE_CASE
- **モジュール**: snake_case

## ファイル構成
- モジュールは`mod.rs`または同名ファイルで定義
- コマンドハンドラーは`src/cli/commands.rs`に集約
- 型定義は`src/core/types.rs`に集約
- プラットフォーム固有コードは`src/platform/`以下

## テスト
- 単体テストは同じファイル内の`#[cfg(test)]`モジュール
- 統合テストは`tests/`ディレクトリ
- モックには`mockall`を使用

## ドキュメント
- 公開APIには`///`ドキュメントコメント
- 内部実装の説明には`//`コメント
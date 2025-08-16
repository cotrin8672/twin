# 開発環境のセットアップ

[English](Getting-Started-Development-Setup) | **日本語**

このガイドでは、Twin の開発環境をセットアップする手順を説明します。

## 前提条件

### 必要なソフトウェア

1. **Rust** (1.70.0 以降)
   ```bash
   # rustup 経由でインストール
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # インストールを確認
   rustc --version
   cargo --version
   ```

2. **Git** (2.15.0 以降 - worktree サポートが必要)
   ```bash
   git --version
   ```

3. **プラットフォーム固有の要件**

   **Windows**:
   - シンボリックリンクサポートのために開発者モードを有効化または管理者として実行
   - Visual Studio Build Tools (コンパイル用)

   **macOS/Linux**:
   - 標準ビルドツール (gcc/clang)

## 開発環境のセットアップ

1. **リポジトリをクローン**
   ```bash
   git clone https://github.com/your-org/twin.git
   cd twin
   ```

2. **依存関係をインストール**
   ```bash
   cargo fetch
   ```

3. **プロジェクトをビルド**
   ```bash
   # デバッグビルド
   cargo build
   
   # リリースビルド
   cargo build --release
   ```

4. **テストを実行**
   ```bash
   # すべてのテスト
   cargo test
   
   # ユニットテストのみ
   cargo test --lib
   
   # 統合テスト
   cargo test --test '*'
   ```

## IDE のセットアップ

### Visual Studio Code
1. Rust Analyzer 拡張機能をインストール
2. TOML Language Support 拡張機能をインストール
3. 推奨設定 (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```

### RustRover/IntelliJ IDEA
1. Rust プラグインをインストール
2. プロジェクトを開いて IDE にインデックスさせる

## 開発ツール

### リンティングとフォーマット
```bash
# コードのフォーマット
cargo fmt

# フォーマットのチェック
cargo fmt --check

# リンターの実行
cargo clippy
```

### デバッグ
プロジェクトは構造化ログのために `tracing` を使用しています：
```bash
# デバッグログを有効化
export TWIN_DEBUG=1
cargo run -- create test-branch

# または RUST_LOG を使用
export RUST_LOG=twin=debug
cargo run -- list
```

ソース: [Cargo.toml#L1-37](https://github.com/your-org/twin/blob/main/Cargo.toml#L1-37)
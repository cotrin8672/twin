# 開発用コマンド一覧

## ビルド・実行
- `cargo build`: プロジェクトをビルド
- `cargo build --release`: リリースビルド
- `cargo run -- [args]`: アプリケーションを実行
- `cargo run -- create [name]`: 新しい環境を作成
- `cargo run -- list`: 環境一覧を表示
- `cargo run -- remove [name]`: 環境を削除
- `cargo run -- config`: 設定管理

## テスト
- `cargo test`: 全テストを実行
- `cargo test --lib`: ライブラリテストのみ
- `cargo test --test integration_test`: 統合テストを実行
- `cargo test -- --nocapture`: 出力を表示しながらテスト

## コード品質
- `cargo fmt`: コードフォーマット
- `cargo fmt --check`: フォーマットチェック（変更なし）
- `cargo clippy`: Lintチェック
- `cargo clippy --fix`: Lint修正を自動適用

## デバッグ
- `RUST_LOG=debug cargo run`: デバッグログを有効化
- `TWIN_DEBUG=1 cargo run`: Twinデバッグモード
- `TWIN_VERBOSE=1 cargo run`: 詳細出力モード

## Git操作（Windows）
- `git status`: 変更状態を確認
- `git add .`: 全変更をステージング
- `git commit -m "message"`: コミット
- `git branch`: ブランチ一覧
- `git checkout -b branch-name`: 新規ブランチ作成

## Windows固有コマンド
- `dir`: ディレクトリ内容表示（lsの代替）
- `type file.txt`: ファイル内容表示（catの代替）
- `findstr pattern file`: パターン検索（grepの代替）
- `where command`: コマンドの場所を検索（whichの代替）
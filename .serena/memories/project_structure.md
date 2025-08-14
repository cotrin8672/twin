# プロジェクト構造

## ディレクトリ構成
```
twin/
├── src/
│   ├── main.rs              # エントリーポイント
│   ├── cli.rs               # CLIパーサー定義
│   ├── cli/
│   │   ├── commands.rs      # コマンドハンドラー実装
│   │   └── output.rs        # 出力フォーマット
│   ├── core/
│   │   ├── mod.rs           # コアモジュール
│   │   ├── types.rs         # 型定義（環境、設定、フック等）
│   │   └── error.rs         # エラー定義
│   ├── platform/            # プラットフォーム固有実装
│   ├── utils/               # ユーティリティ
│   ├── config.rs            # 設定管理
│   ├── environment.rs       # 環境管理
│   ├── git.rs               # Git操作
│   ├── hooks.rs             # フック処理
│   ├── symlink.rs           # シンボリックリンク管理
│   └── tui.rs               # TUIインターフェース（未実装）
├── tests/
│   ├── integration_test.rs  # 統合テスト
│   └── e2e_basic.rs         # E2Eテスト
├── Cargo.toml               # プロジェクト設定
├── Cargo.lock               # 依存関係ロック
└── CLAUDE.md                # Claude Code用指示書
```

## 主要コンポーネント
- **CLI層**: コマンドライン引数の解析と処理
- **Core層**: ビジネスロジックと型定義
- **Platform層**: OS固有の実装
- **Utils層**: 共通ユーティリティ関数

## データフロー
1. `main.rs`でCLI引数を解析
2. `cli/commands.rs`の対応ハンドラーを呼び出し
3. `environment.rs`や`config.rs`でビジネスロジック実行
4. `git.rs`や`symlink.rs`で具体的な操作を実行
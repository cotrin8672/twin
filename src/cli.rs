// This file is kept for backward compatibility
// The actual CLI implementation is now in cli/mod.rs

pub mod commands;
mod output;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// CLIのメインエントリーポイント
/// clapのderiveマクロを使って自動的にコマンドライン引数をパース
#[derive(Parser)]
#[command(name = "twin")]
#[command(about = "Git worktree and symlink environment manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// 利用可能なサブコマンドの定義
#[derive(Subcommand)]
pub enum Commands {
    /// 新しいエージェント環境を作成
    Create(CreateArgs),

    /// 全てのエージェント環境をリスト表示
    List(ListArgs),

    /// エージェント環境を削除
    Remove(RemoveArgs),

    /// 設定を管理
    Config(ConfigArgs),

    /// TUIインターフェースを起動
    Tui,
}

/// createコマンドの引数
#[derive(Parser)]
pub struct CreateArgs {
    /// エージェント名（例: agent-1, feature-x）
    pub agent_name: String,

    /// ブランチ名（指定しない場合はエージェント名から自動生成）
    #[arg(short, long)]
    pub branch: Option<String>,

    /// 設定ファイルのパス
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// 作成後にパスを表示
    #[arg(long)]
    pub print_path: bool,

    /// 作成後にcdコマンドを表示
    #[arg(long)]
    pub cd_command: bool,
}

/// listコマンドの引数
#[derive(Parser)]
pub struct ListArgs {
    /// 出力フォーマット (table, json, simple)
    #[arg(short, long, default_value = "table")]
    pub format: String,
}

/// removeコマンドの引数
#[derive(Parser)]
pub struct RemoveArgs {
    /// 削除するエージェント名
    pub agent_name: String,

    /// 確認なしで強制削除
    #[arg(short, long)]
    pub force: bool,
}

/// configコマンドの引数
#[derive(Parser)]
pub struct ConfigArgs {
    /// 現在の設定を表示
    #[arg(long)]
    pub show: bool,

    /// 設定値をセット (key=value形式)
    #[arg(long)]
    pub set: Option<String>,

    /// 設定値を取得
    #[arg(long)]
    pub get: Option<String>,
}

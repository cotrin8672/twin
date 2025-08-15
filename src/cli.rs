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
    /// ワークツリーを追加（git worktree add）
    Add(AddArgs),

    /// ワークツリーを追加（addのエイリアス、後方互換性のため）
    Create(AddArgs),

    /// 全てのワークツリーをリスト表示
    #[command(alias = "ls")]
    List(ListArgs),

    /// ワークツリーを削除
    #[command(alias = "delete")]
    Remove(RemoveArgs),

    /// 設定を管理
    Config(ConfigArgs),

    /// TUIインターフェースを起動
    Tui,
}

/// addコマンドの引数（git worktree addと互換）
#[derive(Parser)]
pub struct AddArgs {
    /// ワークツリーのパス
    pub path: PathBuf,

    /// ブランチ名（省略時はパスから推測）
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

/// removeコマンドの引数（git worktree removeと互換）
#[derive(Parser)]
pub struct RemoveArgs {
    /// 削除するワークツリーのパスまたは名前
    pub worktree: String,

    /// 確認なしで強制削除
    #[arg(short, long)]
    pub force: bool,
}

/// configコマンドの引数
#[derive(Parser)]
pub struct ConfigArgs {
    /// サブコマンド（default, show, etc）
    pub subcommand: Option<String>,

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

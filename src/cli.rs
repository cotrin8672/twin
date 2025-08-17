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
    /// ワークツリーを追加（デフォルトで新規ブランチを作成）
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

    /// 設定ファイルを初期化
    Init(InitArgs),
}

/// addコマンドの引数（twin独自の使いやすい順序）
#[derive(Parser)]
pub struct AddArgs {
    /// ブランチ名またはコミット
    pub branch: String,

    /// ワークツリーのパス（省略時は設定のworktree_base/ブランチ名）
    pub path: Option<PathBuf>,

    /// 新しいブランチを作成
    #[arg(short = 'b', long)]
    pub new_branch: Option<String>,

    /// 新しいブランチを強制的に作成
    #[arg(short = 'B', long)]
    pub force_branch: Option<String>,

    /// デタッチモード
    #[arg(short = 'd', long)]
    pub detach: bool,

    /// ロックする
    #[arg(long)]
    pub lock: bool,

    /// 追跡モードを設定
    #[arg(long)]
    pub track: bool,

    /// 追跡モードを無効
    #[arg(long)]
    pub no_track: bool,

    /// リモートブランチを推測
    #[arg(long)]
    pub guess_remote: bool,

    /// リモートブランチを推測しない
    #[arg(long)]
    pub no_guess_remote: bool,

    /// チェックアウトしない
    #[arg(long)]
    pub no_checkout: bool,

    /// quietモード
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// twin固有: 設定ファイルのパス
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    /// twin固有: 作成後にパスを表示
    #[arg(long)]
    pub print_path: bool,

    /// twin固有: 作成後にcdコマンドを表示
    #[arg(long)]
    pub cd_command: bool,

    /// twin固有: 副作用をスキップしてgit worktreeのみ実行
    #[arg(long)]
    pub git_only: bool,

    /// twin固有: ブランチの新規作成を無効化（既存ブランチのみ使用）
    #[arg(long, help = "既存のブランチのみを使用し、新規ブランチを作成しない")]
    pub no_create: bool,
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

    /// 設定ファイルのパス
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// git worktree removeのみ実行（副作用をスキップ）
    #[arg(long)]
    pub git_only: bool,

    /// 出力を抑制
    #[arg(short, long)]
    pub quiet: bool,
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

/// initコマンドの引数
#[derive(Parser)]
pub struct InitArgs {
    /// 設定ファイルのパス（デフォルト: twin.toml）
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// 既存のファイルを上書き
    #[arg(short, long)]
    pub force: bool,
}

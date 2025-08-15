use crate::cli::output::OutputFormatter;
use crate::cli::*;
use crate::core::{Config, TwinResult};

// 後方互換性のためのcreateコマンドハンドラー
pub async fn handle_create(args: AddArgs) -> TwinResult<()> {
    handle_add(args).await
}

pub async fn handle_add(args: AddArgs) -> TwinResult<()> {
    use crate::git::GitManager;
    use crate::symlink::create_symlink_manager;

    // 設定を読み込む
    let config = if let Some(config_path) = &args.config {
        Config::from_path(config_path)?
    } else {
        Config::new()
    };

    // worktreeのパスを決定
    let worktree_path = if args.path.is_relative() {
        std::env::current_dir()?.join(&args.path)
    } else {
        args.path.clone()
    };

    // ブランチ名を決定（省略時はパスから推測）
    let branch_name = args.branch.unwrap_or_else(|| {
        args.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("new-branch")
            .to_string()
    });

    // Git worktreeを作成
    let mut git = GitManager::new(std::path::Path::new("."))?;
    let worktree_info = git.add_worktree(&worktree_path, Some(&branch_name), true)?;

    // シンボリックリンクを作成
    if !config.settings.files.is_empty() {
        let symlink_manager = create_symlink_manager();
        let repo_root = git.get_repo_path();

        for mapping in &config.settings.files {
            let source = repo_root.join(&mapping.path);
            let target = worktree_path.join(&mapping.path);

            // ターゲットディレクトリを作成
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // シンボリックリンクを作成
            symlink_manager.create_symlink(&source, &target)?;
        }
    }

    // パス表示やcdコマンド表示の処理
    if args.print_path {
        println!("{}", worktree_path.display());
    } else if args.cd_command {
        println!("cd \"{}\"", worktree_path.display());
    } else {
        println!("✓ Worktree '{}' を作成しました", branch_name);
        println!("  Path: {}", worktree_path.display());
        println!("  Branch: {}", worktree_info.branch);
    }

    Ok(())
}

pub async fn handle_list(args: ListArgs) -> TwinResult<()> {
    use crate::git::GitManager;

    // git worktree list を使用
    let mut git = GitManager::new(std::path::Path::new("."))?;
    let worktrees = git.list_worktrees()?;

    let formatter = OutputFormatter::new(&args.format);
    formatter.format_worktrees(&worktrees)?;

    Ok(())
}

pub async fn handle_remove(args: RemoveArgs) -> TwinResult<()> {
    use crate::git::GitManager;
    use std::path::PathBuf;

    // Worktreeのパスかブランチ名で削除
    let mut git = GitManager::new(std::path::Path::new("."))?;

    // まずworktree一覧を取得して、対応するパスを探す
    let worktrees = git.list_worktrees()?;
    let worktree = worktrees.iter().find(|w| {
        w.branch == args.worktree
            || w.path.file_name().map(|n| n.to_string_lossy()) == Some(args.worktree.clone().into())
            || w.path.to_string_lossy() == args.worktree
    });

    let path = if let Some(wt) = worktree {
        wt.path.clone()
    } else {
        // パスとして解釈してみる
        PathBuf::from(&args.worktree)
    };

    // 確認プロンプト
    if !args.force {
        use std::io::{self, Write};
        print!("Worktree '{}' を削除しますか？ [y/N]: ", path.display());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("削除をキャンセルしました");
            return Ok(());
        }
    }

    // git worktree remove を実行
    git.remove_worktree(&path, args.force)?;
    println!("✓ Worktree '{}' を削除しました", path.display());

    Ok(())
}

pub async fn handle_config(args: ConfigArgs) -> TwinResult<()> {
    use std::path::PathBuf;

    // 設定ファイルのパスを決定
    let config_path = PathBuf::from(".twin.toml");

    // サブコマンドの処理
    if let Some(subcommand) = &args.subcommand {
        match subcommand.as_str() {
            "default" => {
                // デフォルト設定をTOML形式で出力（コメント付き）
                println!("# Twin設定ファイル (.twin.toml)");
                println!("# このファイルをプロジェクトルートに配置してください");
                println!();
                println!("# Worktreeのベースディレクトリ（省略時: ../ブランチ名）");
                println!("# worktree_base = \"../workspaces\"");
                println!();
                println!("# ファイルマッピング設定");
                println!("# Worktree作成時に自動的にシンボリックリンクやコピーを作成します");
                println!("# [[files]]");
                println!("# path = \".env.template\"          # ソースファイルのパス");
                println!("# mapping_type = \"copy\"           # \"symlink\" または \"copy\"");
                println!("# description = \"環境変数設定\"     # 説明（省略可）");
                println!("# skip_if_exists = true           # 既存ファイルをスキップ（省略可）");
                println!();
                println!("# [[files]]");
                println!("# path = \".claude/config.json\"");
                println!("# mapping_type = \"symlink\"");
                println!();
                println!("# フック設定（環境作成・削除時に実行するコマンド）");
                println!("[hooks]");
                println!("# pre_create = [");
                println!("#   {{ command = \"echo\", args = [\"Creating: {{branch}}\"] }}");
                println!("# ]");
                println!("# post_create = [");
                println!(
                    "#   {{ command = \"npm\", args = [\"install\"], continue_on_error = true }}"
                );
                println!("# ]");
                println!("# pre_remove = []");
                println!("# post_remove = []");

                return Ok(());
            }
            _ => {
                println!("不明なサブコマンド: {}", subcommand);
                return Ok(());
            }
        }
    }

    if args.show {
        // 現在の設定を表示
        if config_path.exists() {
            let config = Config::from_path(&config_path)?;
            println!("{:#?}", config);
        } else {
            println!("設定ファイルが見つかりません: {}", config_path.display());
        }
    } else if let Some(set_value) = args.set {
        // 設定値をセット (key=value形式)
        let parts: Vec<&str> = set_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(crate::core::error::TwinError::Config {
                message: "設定値は 'key=value' 形式で指定してください".to_string(),
                path: None,
                source: None,
            });
        }

        println!("設定 '{}' を '{}' に設定しました", parts[0], parts[1]);
        println!("注: この機能は現在実装中です");
    } else if let Some(key) = args.get {
        // 設定値を取得
        if config_path.exists() {
            let _config = Config::from_path(&config_path)?;
            println!("キー '{}' の値を取得します", key);
            println!("注: この機能は現在実装中です");
        } else {
            println!("設定ファイルが見つかりません: {}", config_path.display());
        }
    } else {
        println!("使用方法:");
        println!("  twin config default         : デフォルト設定をTOML形式で出力");
        println!("  twin config --show          : 現在の設定を表示");
        println!("  twin config --set key=value : 設定値をセット");
        println!("  twin config --get key       : 設定値を取得");
    }

    Ok(())
}

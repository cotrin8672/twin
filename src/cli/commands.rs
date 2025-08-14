use crate::cli::output::OutputFormatter;
use crate::cli::*;
use crate::core::{Config, TwinResult};

pub async fn handle_create(args: CreateArgs) -> TwinResult<()> {
    use crate::git::GitManager;
    use crate::symlink::create_symlink_manager;
    
    // 設定を読み込む
    let config = if let Some(config_path) = &args.config {
        Config::from_path(config_path)?
    } else {
        Config::new()
    };

    // ディレクトリの決定
    // 優先順位: 1. CLI引数 2. 設定ファイル 3. デフォルト(../branch_name)
    let worktree_dir = if let Some(dir) = args.directory {
        dir
    } else if let Some(base) = &config.settings.worktree_base {
        base.join(&args.branch_name)
    } else {
        // デフォルト: 親ディレクトリにブランチ名のディレクトリを作成
        std::path::PathBuf::from("..").join(&args.branch_name)
    };

    // 絶対パスに変換
    let worktree_path = if worktree_dir.is_relative() {
        std::env::current_dir()?.join(&worktree_dir)
    } else {
        worktree_dir
    };

    // Git worktreeを作成
    let mut git = GitManager::new(std::path::Path::new("."))?;
    let worktree_info = git.add_worktree(&worktree_path, Some(&args.branch_name), true)?;
    
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
        println!("✓ Worktree '{}' を作成しました", args.branch_name);
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
        w.branch == args.branch_name || 
        w.path.file_name().map(|n| n.to_string_lossy()) == Some(args.branch_name.clone().into())
    });
    
    let path = if let Some(wt) = worktree {
        wt.path.clone()
    } else {
        // パスとして解釈してみる
        PathBuf::from(&args.branch_name)
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
        println!("  twin config --show          : 現在の設定を表示");
        println!("  twin config --set key=value : 設定値をセット");
        println!("  twin config --get key       : 設定値を取得");
    }

    Ok(())
}

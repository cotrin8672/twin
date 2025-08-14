use crate::cli::output::OutputFormatter;
use crate::cli::*;
use crate::core::{Config, TwinResult};
use crate::environment::EnvironmentManager;

pub async fn handle_create(args: CreateArgs) -> TwinResult<()> {
    // 設定を読み込む
    let config = if let Some(config_path) = &args.config {
        Config::from_path(config_path)?
    } else {
        Config::new()
    };

    let mut manager = EnvironmentManager::new(config.clone())?;

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

    // 環境を作成
    let env = manager.create_environment(args.branch_name.clone(), worktree_dir)?;

    // パス表示やcdコマンド表示の処理
    if args.print_path {
        println!("{}", env.worktree_path.display());
    } else if args.cd_command {
        println!("cd \"{}\"", env.worktree_path.display());
    } else {
        println!("✓ 環境 '{}' を作成しました", args.branch_name);
        println!("  Worktree: {}", env.worktree_path.display());
        println!("  Branch: {}", env.branch);
    }

    Ok(())
}

pub async fn handle_list(args: ListArgs) -> TwinResult<()> {
    let config = Config::new();
    let manager = EnvironmentManager::new(config)?;
    let environments = manager.list_environments_from_registry();

    let formatter = OutputFormatter::new(&args.format);

    // Vec<&AgentEnvironment> を Vec<AgentEnvironment> に変換
    let environments_owned: Vec<_> = environments.into_iter().cloned().collect();
    formatter.format_environments(&environments_owned)?;

    Ok(())
}

pub async fn handle_remove(args: RemoveArgs) -> TwinResult<()> {
    let config = Config::new();
    let mut manager = EnvironmentManager::new(config)?;

    // 確認プロンプト
    if !args.force {
        use std::io::{self, Write};
        print!("環境 '{}' を削除しますか？ [y/N]: ", args.branch_name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("削除をキャンセルしました");
            return Ok(());
        }
    }

    manager.remove_environment(&args.branch_name, args.force)?;
    println!("✓ 環境 '{}' を削除しました", args.branch_name);

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

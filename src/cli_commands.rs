use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

use crate::cli::{CreateArgs, ListArgs, RemoveArgs, SwitchArgs, InitArgs, ConfigArgs};
use crate::cli::output::{OutputFormat, format_environments, format_path_output};
use crate::config::Config;
use crate::core::{AgentEnvironment, EnvironmentRegistry, EnvironmentStatus};
use crate::environment::EnvironmentManager;

/// createコマンドの実装
pub async fn handle_create(args: CreateArgs) -> Result<()> {
    println!("Creating environment: {}", args.agent_name);
    
    // 設定を読み込み
    let config = if let Some(config_path) = &args.config {
        Config::load(config_path).await
            .with_context(|| format!("Failed to load config from {}", config_path.display()))?
    } else {
        Config::load_or_default(Some(&env::current_dir()?)).await?
    };
    
    // 環境マネージャーを初期化
    let manager = EnvironmentManager::new();
    
    // TODO: 実際の環境作成処理を実装
    // 現在はモックデータで動作確認
    let worktree_path = env::current_dir()?.join("worktrees").join(&args.agent_name);
    let branch = args.branch.unwrap_or_else(|| format!("agent/{}", args.agent_name));
    
    let mut env = AgentEnvironment::new(
        args.agent_name.clone(),
        branch,
        worktree_path.clone(),
        args.config,
    );
    env.set_status(EnvironmentStatus::Active);
    
    println!("✓ Environment '{}' created successfully", args.agent_name);
    
    // パス出力オプション
    if args.print_path {
        println!("{}", worktree_path.display());
    }
    
    // cdコマンド出力オプション
    if args.cd_command {
        format_path_output(&worktree_path, true)?;
    }
    
    Ok(())
}

/// listコマンドの実装
pub async fn handle_list(args: ListArgs) -> Result<()> {
    // 出力フォーマットを解析
    let format = OutputFormat::from_str(&args.format)?;
    
    // TODO: 実際の環境レジストリから読み込み
    // 現在はモックデータで動作確認
    let mut registry = EnvironmentRegistry::new();
    
    // サンプル環境を追加
    let env1 = AgentEnvironment::new(
        "agent-001".to_string(),
        "agent/feature-auth".to_string(),
        PathBuf::from("./worktrees/agent-001"),
        None,
    );
    
    let mut env2 = AgentEnvironment::new(
        "agent-002".to_string(),
        "agent/bugfix-ui".to_string(),
        PathBuf::from("./worktrees/agent-002"),
        None,
    );
    env2.set_status(EnvironmentStatus::Inactive);
    
    registry.add(env1);
    registry.add(env2);
    registry.set_active(Some("agent-001".to_string()));
    
    // 環境一覧を出力
    let environments: Vec<&AgentEnvironment> = registry.environments.values().collect();
    format_environments(&environments, &format, registry.active.as_deref())?;
    
    Ok(())
}

/// removeコマンドの実装
pub async fn handle_remove(args: RemoveArgs) -> Result<()> {
    println!("Removing environment: {}", args.agent_name);
    
    // 確認プロンプト（forceオプションが指定されていない場合）
    if !args.force {
        println!("This will permanently delete the environment and all its data.");
        println!("Are you sure you want to continue? (y/N)");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        if input != "y" && input != "yes" {
            println!("Operation cancelled.");
            return Ok(());
        }
    }
    
    // TODO: 実際の削除処理を実装
    println!("✓ Environment '{}' removed successfully", args.agent_name);
    
    Ok(())
}

/// switchコマンドの実装
pub async fn handle_switch(args: SwitchArgs) -> Result<()> {
    println!("Switching to environment: {}", args.agent_name);
    
    // TODO: 実際の切り替え処理を実装
    let worktree_path = env::current_dir()?.join("worktrees").join(&args.agent_name);
    
    println!("✓ Switched to environment '{}'", args.agent_name);
    
    // パス出力オプション
    if args.print_path {
        println!("{}", worktree_path.display());
    }
    
    // cdコマンド出力オプション
    if args.cd_command {
        format_path_output(&worktree_path, true)?;
    }
    
    Ok(())
}

/// initコマンドの実装
pub async fn handle_init(args: InitArgs) -> Result<()> {
    let config_path = Config::init(args.path, args.force).await?;
    
    println!("✓ Configuration file created: {}", config_path.display());
    println!();
    println!("You can now edit the configuration file to customize:");
    println!("  - Symlink targets");
    println!("  - Hook commands");
    println!("  - Worktree base directory");
    println!("  - Branch naming prefix");
    
    Ok(())
}

/// configコマンドの実装
pub async fn handle_config(args: ConfigArgs) -> Result<()> {
    if args.show {
        // 現在の設定を表示
        let config = Config::load_or_default(Some(&env::current_dir()?)).await?;
        
        println!("Current configuration:");
        println!("{}", toml::to_string_pretty(&config)?);
        
        return Ok(());
    }
    
    if let Some(key) = args.get {
        // 設定値を取得
        println!("Getting configuration value for key: {}", key);
        // TODO: 実装
        return Ok(());
    }
    
    if let Some(key_value) = args.set {
        // 設定値をセット
        println!("Setting configuration: {}", key_value);
        // TODO: 実装
        return Ok(());
    }
    
    // オプションが指定されていない場合はヘルプを表示
    println!("Use --show to display current configuration");
    println!("Use --get <key> to get a specific value");
    println!("Use --set <key=value> to set a specific value");
    
    Ok(())
}
use crate::cli::output::OutputFormatter;
use crate::cli::*;
use crate::core::{Config, TwinError, TwinResult};

// 後方互換性のためのcreateコマンドハンドラー
pub async fn handle_create(args: AddArgs) -> TwinResult<()> {
    handle_add(args).await
}

pub async fn handle_add(args: AddArgs) -> TwinResult<()> {
    use crate::git::GitManager;
    use crate::hooks::{HookContext, HookExecutor, HookType};
    use crate::symlink::create_symlink_manager;
    use std::path::PathBuf;

    // 設定を読み込む
    let config = if let Some(config_path) = &args.config {
        Config::from_path(config_path)?
    } else {
        Config::new()
    };

    // ワークツリーのパスを決定
    // パスが指定されていない場合は、worktree_base設定を使用
    let worktree_path = if let Some(path) = &args.path {
        path.clone()
    } else {
        // ブランチ名からディレクトリ名を作成（スラッシュをハイフンに置換）
        let dir_name = args.branch.replace('/', "-");

        // worktree_baseが設定されていればそれを使用、なければデフォルト
        if let Some(base) = &config.settings.worktree_base {
            base.join(&dir_name)
        } else {
            // デフォルトは ./worktrees/ブランチ名
            PathBuf::from("worktrees").join(&dir_name)
        }
    };

    // Git worktreeを作成
    let mut git = GitManager::new(std::path::Path::new("."))?;

    // git worktree addの引数を構築
    let mut worktree_args = Vec::new();

    // ブランチが存在するかチェック
    let branch_exists = git.branch_exists(&args.branch)?;

    // オプションを追加
    if let Some(branch) = &args.new_branch {
        worktree_args.push("-b");
        worktree_args.push(branch.as_str());
    } else if let Some(branch) = &args.force_branch {
        worktree_args.push("-B");
        worktree_args.push(branch.as_str());
    } else if !branch_exists && !args.detach {
        // ブランチが存在しない場合は自動的に-bオプションを追加
        worktree_args.push("-b");
        worktree_args.push(args.branch.as_str());
    }
    if args.detach {
        worktree_args.push("--detach");
    }
    if args.lock {
        worktree_args.push("--lock");
    }
    if args.track {
        worktree_args.push("--track");
    }
    if args.no_track {
        worktree_args.push("--no-track");
    }
    if args.guess_remote {
        worktree_args.push("--guess-remote");
    }
    if args.no_guess_remote {
        worktree_args.push("--no-guess-remote");
    }
    if args.no_checkout {
        worktree_args.push("--no-checkout");
    }
    if args.quiet {
        worktree_args.push("--quiet");
    }

    // パスを追加
    let path_str = worktree_path.to_string_lossy();
    worktree_args.push(&path_str);

    // ブランチ/コミットを追加
    let branch_str = args.branch.clone();

    // 新規ブランチ作成の場合、ブランチ参照は-b/-Bオプションで既に指定済み
    // detachモードの場合、HEADをブランチ参照として使用
    if args.new_branch.is_none() && args.force_branch.is_none() {
        if !branch_exists && !args.detach {
            // ブランチが存在しない場合（既に-bオプションを追加済み）
            // ブランチ参照は不要
        } else if args.detach {
            // detachモードの場合、HEADを使用
            worktree_args.push("HEAD");
        } else {
            // 既存のブランチを参照
            worktree_args.push(&branch_str);
        }
    }

    // worktreeのパスを正規化（絶対パスに）
    let worktree_path_absolute = if worktree_path.is_relative() {
        std::env::current_dir()?
            .join(&worktree_path)
            .canonicalize()
            .unwrap_or_else(|_| {
                // canonicalizeが失敗した場合（まだ存在しないパスの場合）
                let cwd = std::env::current_dir().unwrap();
                let mut result = cwd.clone();
                for component in worktree_path.components() {
                    match component {
                        std::path::Component::ParentDir => {
                            result.pop();
                        }
                        std::path::Component::Normal(name) => {
                            result.push(name);
                        }
                        _ => {}
                    }
                }
                result
            })
    } else {
        worktree_path.clone()
    };

    // git_onlyモードの場合は副作用をスキップ
    if args.git_only {
        let output = git.add_worktree_with_options(&worktree_args)?;
        if !args.quiet {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        return Ok(());
    }

    // ブランチ名を決定
    let branch_name = args
        .new_branch
        .as_ref()
        .or(args.force_branch.as_ref())
        .cloned()
        .unwrap_or_else(|| args.branch.clone());

    // フック実行の準備
    let hook_executor = HookExecutor::new();
    let hook_context = HookContext::new(
        branch_name.clone(), // agent_nameの代わりにブランチ名を使用
        worktree_path_absolute.clone(),
        branch_name.clone(),
        git.get_repo_path().to_path_buf(),
    );

    // pre_createフックを実行
    if !config.settings.hooks.pre_create.is_empty() {
        for hook in &config.settings.hooks.pre_create {
            match hook_executor.execute(HookType::PreCreate, hook, &hook_context) {
                Ok(result) => {
                    if !result.success && !hook.continue_on_error {
                        return Err(TwinError::hook(
                            format!("Pre-create hook failed: {}", hook.command),
                            "pre_create",
                            result.exit_code,
                        ));
                    }
                }
                Err(e) if !hook.continue_on_error => return Err(e),
                Err(e) => eprintln!("Warning: Pre-create hook failed: {e}"),
            }
        }
    }

    // 通常モード: git worktreeを実行して副作用を適用
    let output = git.add_worktree_with_options(&worktree_args)?;
    let _worktree_info = git.get_worktree_info(&worktree_path)?;

    // シンボリックリンクを作成（副作用）
    if !config.settings.files.is_empty() && !args.git_only {
        let symlink_manager = create_symlink_manager();
        let repo_root = git.get_repo_path();
        let mut failed_links = Vec::new();

        for mapping in &config.settings.files {
            // ソースは絶対パスに変換（repo_rootが"."の場合は現在のディレクトリを使用）
            let source = if repo_root == std::path::Path::new(".") {
                std::env::current_dir()?.join(&mapping.path)
            } else if repo_root.is_absolute() {
                repo_root.join(&mapping.path)
            } else {
                std::env::current_dir()?.join(repo_root).join(&mapping.path)
            };
            let target = worktree_path_absolute.join(&mapping.path);

            // ソースファイルが存在しない場合はスキップ
            if !source.exists() {
                eprintln!(
                    "⚠️  Warning: Source file not found, skipping: {}",
                    source.display()
                );
                failed_links.push(mapping.path.clone());
                continue;
            }

            // ターゲットディレクトリを作成
            if let Some(parent) = target.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!(
                        "⚠️  Warning: Failed to create directory {}: {}",
                        parent.display(),
                        e
                    );
                    failed_links.push(mapping.path.clone());
                    continue;
                }
            }

            // シンボリックリンクを作成（エラー時は警告を表示して継続）
            match symlink_manager.create_symlink(&source, &target) {
                Ok(_) => {
                    if !args.quiet {
                        eprintln!(
                            "✓ Created symlink: {} -> {}",
                            target.display(),
                            source.display()
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "⚠️  Warning: Failed to create symlink for {}: {}",
                        mapping.path.display(),
                        e
                    );
                    failed_links.push(mapping.path.clone());
                }
            }
        }

        // 失敗したリンクがある場合の警告
        if !failed_links.is_empty() && !args.quiet {
            eprintln!("⚠️  {} symlink(s) could not be created", failed_links.len());
            eprintln!("   The worktree was created successfully, but some symlinks failed.");
        }
    }

    // post_createフックを実行
    if !config.settings.hooks.post_create.is_empty() {
        for hook in &config.settings.hooks.post_create {
            match hook_executor.execute(HookType::PostCreate, hook, &hook_context) {
                Ok(result) => {
                    if !result.success && !hook.continue_on_error {
                        eprintln!("Error: Post-create hook failed: {}", hook.command);
                        // post_createで失敗してもworktreeは既に作成済みなので、警告のみ
                    }
                }
                Err(e) => eprintln!("Warning: Post-create hook failed: {e}"),
            }
        }
    }

    // パス表示やcdコマンド表示の処理
    if args.print_path {
        println!("{}", worktree_path_absolute.display());
    } else if args.cd_command {
        println!("cd \"{}\"", worktree_path_absolute.display());
    } else if !args.quiet {
        // git worktreeの出力をそのまま表示
        print!("{}", String::from_utf8_lossy(&output.stdout));
        if !config.settings.files.is_empty() {
            println!("✓ シンボリックリンクを作成しました");
        }
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
    use crate::hooks::{HookContext, HookExecutor, HookType};
    use crate::symlink::create_symlink_manager;
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

    // 設定を読み込む
    let config = if let Some(config_path) = &args.config {
        Config::from_path(config_path)?
    } else {
        Config::new()
    };

    // フック実行の準備（削除時はブランチ名かパス名を使用）
    let branch_name = worktree.map(|w| w.branch.clone()).unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("worktree")
            .to_string()
    });

    let hook_executor = HookExecutor::new();
    let hook_context = HookContext::new(
        branch_name.clone(),
        path.clone(),
        branch_name.clone(),
        git.get_repo_path().to_path_buf(),
    );

    // pre_removeフックを実行
    if !config.settings.hooks.pre_remove.is_empty() && !args.git_only {
        for hook in &config.settings.hooks.pre_remove {
            match hook_executor.execute(HookType::PreRemove, hook, &hook_context) {
                Ok(result) => {
                    if !result.success && !hook.continue_on_error {
                        return Err(TwinError::hook(
                            format!("Pre-remove hook failed: {}", hook.command),
                            "pre_remove",
                            result.exit_code,
                        ));
                    }
                }
                Err(e) if !hook.continue_on_error => return Err(e),
                Err(e) => eprintln!("Warning: Pre-remove hook failed: {e}"),
            }
        }
    }

    // シンボリックリンクを削除（副作用のクリーンアップ）

    if !config.settings.files.is_empty() && !args.git_only {
        let symlink_manager = create_symlink_manager();
        let mut failed_cleanups = Vec::new();

        for mapping in &config.settings.files {
            let target = path.join(&mapping.path);

            // シンボリックリンクが存在する場合のみ削除
            if target.exists() || target.is_symlink() {
                match symlink_manager.remove_symlink(&target) {
                    Ok(_) => {
                        if !args.quiet {
                            eprintln!("✓ Removed symlink: {}", target.display());
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "⚠️  Warning: Failed to remove symlink {}: {}",
                            target.display(),
                            e
                        );
                        failed_cleanups.push(mapping.path.clone());
                    }
                }
            }
        }

        if !failed_cleanups.is_empty() && !args.quiet {
            eprintln!(
                "⚠️  {} symlink(s) could not be removed",
                failed_cleanups.len()
            );
            eprintln!("   Proceeding with worktree removal anyway.");
        }
    }

    // git worktree remove を実行
    git.remove_worktree(&path, args.force)?;

    // post_removeフックを実行
    if !config.settings.hooks.post_remove.is_empty() && !args.git_only {
        for hook in &config.settings.hooks.post_remove {
            match hook_executor.execute(HookType::PostRemove, hook, &hook_context) {
                Ok(result) => {
                    if !result.success && !hook.continue_on_error {
                        eprintln!("Error: Post-remove hook failed: {}", hook.command);
                        // post_removeで失敗してもworktreeは既に削除済みなので、警告のみ
                    }
                }
                Err(e) => eprintln!("Warning: Post-remove hook failed: {e}"),
            }
        }
    }

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
                println!("不明なサブコマンド: {subcommand}");
                return Ok(());
            }
        }
    }

    if args.show {
        // 現在の設定を表示
        if config_path.exists() {
            let config = Config::from_path(&config_path)?;
            println!("{config:#?}");
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
            println!("キー '{key}' の値を取得します");
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

/// initコマンドのハンドラー
pub async fn handle_init(args: InitArgs) -> TwinResult<()> {
    // TODO(human): Add interactive mode support here
    // When args.interactive is true, prompt user for:
    // - worktree_base (default: "./worktrees")
    // - branch_prefix (default: "agent/")
    // Then pass these values to Config::init_with_options() or similar

    // config::Config::init()を呼び出して設定ファイルを作成
    let config_path = crate::config::Config::init(args.path, args.force).await?;

    println!("✅ 設定ファイルを作成しました: {}", config_path.display());
    println!();
    println!("設定ファイルを編集して、プロジェクトに合わせてカスタマイズできます。");
    println!("主な設定項目:");
    println!("  - worktree_base: ワークツリーのベースディレクトリ");
    println!("  - branch_prefix: ブランチ名のプレフィックス");
    println!("  - files: シンボリックリンク/コピーするファイルマッピング");
    println!("  - hooks: 各種フック（add, remove時の処理）");

    Ok(())
}

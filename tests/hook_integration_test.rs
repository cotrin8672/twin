//! フック機能の統合テスト
use std::fs;
use std::path::Path;
use twin_cli::cli::commands::{handle_add, handle_remove};
use twin_cli::cli::{AddArgs, RemoveArgs};

mod common;
use common::TestRepo;

/// フック実行を記録するためのファイルを作成
fn create_hook_config(_test_dir: &Path) -> String {
    r#"
[[files]]
path = ".env"
mapping_type = "symlink"

[hooks]
pre_create = [
    { command = "echo", args = ["pre_create: {{branch}}"] },
    { command = "echo", args = ["pre_create_to_file"], continue_on_error = true }
]
post_create = [
    { command = "echo", args = ["post_create: {{branch}}"] }
]
pre_remove = [
    { command = "echo", args = ["pre_remove: {{branch}}"] }
]
post_remove = [
    { command = "echo", args = ["post_remove: {{branch}}"] }
]
"#.to_string()
}

#[tokio::test]
async fn test_hooks_execution_on_add() {
    let test_repo = TestRepo::new();
    let test_id = uuid::Uuid::new_v4().to_string()[0..8].to_string();
    let config_path = test_repo.path().join(".twin.toml");

    // フック設定を作成
    let config = create_hook_config(test_repo.path());
    fs::write(&config_path, config).unwrap();

    // .envファイルを作成（シンボリックリンク用）
    let env_file = test_repo.path().join(".env");
    fs::write(&env_file, "TEST=value").unwrap();

    // worktree作成
    let worktree_path = test_repo.path().join("wt-hooks");
    let args = AddArgs {
        path: worktree_path.clone(),
        branch: None,
        new_branch: Some(format!("test-hooks-{test_id}")),
        force_branch: None,
        detach: false,
        config: Some(config_path.clone()),
        git_only: false,
        lock: false,
        track: false,
        no_track: false,
        guess_remote: false,
        no_guess_remote: false,
        no_checkout: false,
        quiet: false,
        print_path: false,
        cd_command: false,
    };

    // フックが実行されることを確認（エラーが出ないこと）
    let result = handle_add(args).await;
    assert!(
        result.is_ok(),
        "Failed to create worktree with hooks: {:?}",
        result.err()
    );

    // worktreeが作成されたことを確認
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // シンボリックリンクが作成されたことを確認
    let symlink = worktree_path.join(".env");
    assert!(symlink.exists() || symlink.is_symlink());
}

#[tokio::test]
async fn test_hooks_execution_on_remove() {
    let test_repo = TestRepo::new();
    let test_id = uuid::Uuid::new_v4().to_string()[0..8].to_string();
    let config_path = test_repo.path().join(".twin.toml");

    // フック設定を作成
    let config = create_hook_config(test_repo.path());
    fs::write(&config_path, config).unwrap();

    // .envファイルを作成
    let env_file = test_repo.path().join(".env");
    fs::write(&env_file, "TEST=value").unwrap();

    // まずworktreeを作成
    let worktree_path = test_repo.path().join("wt-remove-hooks");
    let add_args = AddArgs {
        path: worktree_path.clone(),
        branch: None,
        new_branch: Some(format!("test-remove-{test_id}")),
        force_branch: None,
        detach: false,
        config: Some(config_path.clone()),
        git_only: false,
        lock: false,
        track: false,
        no_track: false,
        guess_remote: false,
        no_guess_remote: false,
        no_checkout: false,
        quiet: true,
        print_path: false,
        cd_command: false,
    };

    let result = handle_add(add_args).await;
    assert!(result.is_ok());
    assert!(worktree_path.exists());

    // worktreeを削除
    let remove_args = RemoveArgs {
        worktree: worktree_path.to_string_lossy().to_string(),
        force: true,
        config: Some(config_path),
        git_only: false,
        quiet: false,
    };

    // フックが実行されることを確認
    let result = handle_remove(remove_args).await;
    assert!(
        result.is_ok(),
        "Failed to remove worktree with hooks: {:?}",
        result.err()
    );

    // worktreeが削除されたことを確認
    assert!(!worktree_path.exists());
}

#[tokio::test]
async fn test_hook_continue_on_error() {
    let test_repo = TestRepo::new();
    let test_id = uuid::Uuid::new_v4().to_string()[0..8].to_string();
    let config_path = test_repo.path().join(".twin.toml");

    // エラーが出るフックだが、continue_on_error=trueなので続行する設定
    let config = r#"
[hooks]
pre_create = [
    { command = "cmd", args = ["/C", "exit", "1"], continue_on_error = true },
    { command = "echo", args = ["This should run"] }
]
"#;
    fs::write(&config_path, config).unwrap();

    let worktree_path = test_repo.path().join("wt-error-continue");
    let args = AddArgs {
        path: worktree_path.clone(),
        branch: None,
        new_branch: Some(format!("test-error-{test_id}")),
        force_branch: None,
        detach: false,
        config: Some(config_path),
        git_only: false,
        lock: false,
        track: false,
        no_track: false,
        guess_remote: false,
        no_guess_remote: false,
        no_checkout: false,
        quiet: false,
        print_path: false,
        cd_command: false,
    };

    // continue_on_error=trueなのでworktree作成は成功するはず
    let result = handle_add(args).await;
    assert!(result.is_ok(), "Should continue despite hook error");
    assert!(worktree_path.exists());
}

#[tokio::test]
async fn test_hook_fail_on_error() {
    let test_repo = TestRepo::new();
    let test_id = uuid::Uuid::new_v4().to_string()[0..8].to_string();
    let config_path = test_repo.path().join(".twin.toml");

    // エラーが出るフックで、continue_on_error=falseなので中断する設定
    let config = r#"
[hooks]
pre_create = [
    { command = "cmd", args = ["/C", "exit", "1"], continue_on_error = false }
]
"#;
    fs::write(&config_path, config).unwrap();

    let worktree_path = test_repo.path().join("wt-error-fail");
    let args = AddArgs {
        path: worktree_path.clone(),
        branch: None,
        new_branch: Some(format!("test-fail-{test_id}")),
        force_branch: None,
        detach: false,
        config: Some(config_path),
        git_only: false,
        lock: false,
        track: false,
        no_track: false,
        guess_remote: false,
        no_guess_remote: false,
        no_checkout: false,
        quiet: false,
        print_path: false,
        cd_command: false,
    };

    // フックが失敗してworktree作成も失敗するはず
    let result = handle_add(args).await;
    assert!(
        result.is_err(),
        "Should fail when hook fails and continue_on_error=false"
    );
    assert!(!worktree_path.exists());
}

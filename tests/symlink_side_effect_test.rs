/// シンボリックリンク副作用のテスト
///
/// worktree作成/削除時のシンボリックリンク副作用をテストします。
mod common;

use common::TestRepo;
use std::fs;

#[test]
fn test_symlink_creation_on_add() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();

    // テスト用の設定ファイルを作成
    let config_content = r#"
[[settings.files]]
path = ".env.template"
description = "環境変数テンプレート"

[[settings.files]]
path = "config/settings.json"
description = "共有設定ファイル"
"#;

    // 設定ファイルとソースファイルを作成
    repo.exec(&[
        "sh",
        "-c",
        &format!("echo '{}' > .twin.toml", config_content),
    ]);
    repo.exec(&["sh", "-c", "echo 'TEST_VAR=value' > .env.template"]);
    repo.exec(&["mkdir", "-p", "config"]);
    repo.exec(&["sh", "-c", "echo '{\"test\": true}' > config/settings.json"]);

    // worktreeを作成（シンボリックリンク副作用が適用される）
    let worktree_path = repo.worktree_path("symlink-test");
    let output = repo.run_twin(&[
        "add",
        &worktree_path,
        "-b",
        "symlink-branch",
        "--config",
        ".twin.toml",
    ]);

    assert!(
        output.status.success(),
        "Failed to add worktree: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    // シンボリックリンクが作成されたことを確認
    let check_env = repo.exec(&[
        "test",
        "-L",
        &format!("{}/{}", worktree_path, ".env.template"),
    ]);
    let check_config = repo.exec(&[
        "test",
        "-L",
        &format!("{}/{}", worktree_path, "config/settings.json"),
    ]);

    // Linuxではシンボリックリンクが作成されるはず
    assert!(
        check_env.status.success() || check_config.status.success(),
        "Symlinks should be created"
    );

    // リンク先が正しいことを確認
    let env_content = repo.exec(&["cat", &format!("{}/{}", worktree_path, ".env.template")]);
    let env_text = String::from_utf8_lossy(&env_content.stdout);
    assert!(
        env_text.contains("TEST_VAR=value"),
        "Symlink should point to correct file"
    );
}

#[test]
fn test_symlink_removal_on_remove() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();

    // 設定ファイルとソースファイルを作成
    let config_content = r#"
[[settings.files]]
path = "shared.txt"
"#;

    repo.exec(&[
        "sh",
        "-c",
        &format!("echo '{}' > .twin.toml", config_content),
    ]);
    repo.exec(&["sh", "-c", "echo 'shared content' > shared.txt"]);

    // worktreeを作成
    let worktree_path = repo.worktree_path("remove-test");
    let output = repo.run_twin(&[
        "add",
        &worktree_path,
        "-b",
        "remove-branch",
        "--config",
        ".twin.toml",
    ]);
    assert!(output.status.success());

    // シンボリックリンクが存在することを確認
    let symlink_path = format!("{}/shared.txt", worktree_path);
    let check = repo.exec(&["test", "-e", &symlink_path]);
    assert!(
        check.status.success(),
        "Symlink should exist after creation"
    );

    // worktreeを削除（シンボリックリンクも削除される）
    let output = repo.run_twin(&[
        "remove",
        &worktree_path,
        "--force",
        "--config",
        ".twin.toml",
    ]);
    assert!(output.status.success(), "Failed to remove worktree");

    // worktreeディレクトリが削除されたことを確認
    let check = repo.exec(&["test", "-d", &worktree_path]);
    assert!(
        !check.status.success(),
        "Worktree directory should be removed"
    );
}

#[test]
fn test_git_only_mode_skips_symlinks() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();

    // 設定ファイルを作成
    let config_content = r#"
[[settings.files]]
path = "should-not-exist.txt"
"#;

    repo.exec(&[
        "sh",
        "-c",
        &format!("echo '{}' > .twin.toml", config_content),
    ]);
    repo.exec(&["sh", "-c", "echo 'source' > should-not-exist.txt"]);

    // --git-onlyモードでworktreeを作成
    let worktree_path = repo.worktree_path("git-only");
    let output = repo.run_twin(&[
        "add",
        &worktree_path,
        "-b",
        "git-only-branch",
        "--config",
        ".twin.toml",
        "--git-only",
    ]);

    assert!(output.status.success());

    // シンボリックリンクが作成されていないことを確認
    let symlink_path = format!("{}/should-not-exist.txt", worktree_path);
    let check = repo.exec(&["test", "-L", &symlink_path]);
    assert!(
        !check.status.success(),
        "Symlink should not be created with --git-only"
    );
}

#[test]
fn test_symlink_error_handling() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();

    // 存在しないファイルへのシンボリックリンクを設定
    let config_content = r#"
[[settings.files]]
path = "nonexistent.txt"

[[settings.files]]
path = "exists.txt"
"#;

    repo.exec(&[
        "sh",
        "-c",
        &format!("echo '{}' > .twin.toml", config_content),
    ]);
    repo.exec(&["sh", "-c", "echo 'content' > exists.txt"]);

    // worktreeを作成（一部のシンボリックリンクは失敗する）
    let worktree_path = repo.worktree_path("error-test");
    let output = repo.run_twin(&[
        "add",
        &worktree_path,
        "-b",
        "error-branch",
        "--config",
        ".twin.toml",
    ]);

    // worktree作成自体は成功するはず
    assert!(
        output.status.success(),
        "Worktree creation should succeed despite symlink errors"
    );

    // エラーメッセージが表示されることを確認
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning") || stderr.contains("not found"),
        "Should warn about missing source file"
    );

    // 存在するファイルのシンボリックリンクは作成される
    let exists_path = format!("{}/exists.txt", worktree_path);
    let check = repo.exec(&["test", "-e", &exists_path]);
    assert!(check.status.success(), "Valid symlink should be created");

    // 存在しないファイルのシンボリックリンクは作成されない
    let nonexistent_path = format!("{}/nonexistent.txt", worktree_path);
    let check = repo.exec(&["test", "-e", &nonexistent_path]);
    assert!(
        !check.status.success(),
        "Invalid symlink should not be created"
    );
}

#[test]
fn test_symlink_with_subdirectories() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();

    // ネストしたディレクトリ構造のファイルを設定
    let config_content = r#"
[[settings.files]]
path = "src/config/app.json"

[[settings.files]]
path = "data/cache/index.bin"
"#;

    repo.exec(&[
        "sh",
        "-c",
        &format!("echo '{}' > .twin.toml", config_content),
    ]);
    repo.exec(&["mkdir", "-p", "src/config"]);
    repo.exec(&["mkdir", "-p", "data/cache"]);
    repo.exec(&["sh", "-c", "echo '{\"app\": true}' > src/config/app.json"]);
    repo.exec(&["sh", "-c", "echo 'binary data' > data/cache/index.bin"]);

    // worktreeを作成
    let worktree_path = repo.worktree_path("nested-test");
    let output = repo.run_twin(&[
        "add",
        &worktree_path,
        "-b",
        "nested-branch",
        "--config",
        ".twin.toml",
    ]);

    assert!(output.status.success());

    // ディレクトリ構造が作成されてシンボリックリンクが存在することを確認
    let app_path = format!("{}/src/config/app.json", worktree_path);
    let cache_path = format!("{}/data/cache/index.bin", worktree_path);

    let check_app = repo.exec(&["test", "-L", &app_path]);
    let check_cache = repo.exec(&["test", "-L", &cache_path]);

    assert!(
        check_app.status.success() || check_cache.status.success(),
        "Nested symlinks should be created with proper directory structure"
    );
}

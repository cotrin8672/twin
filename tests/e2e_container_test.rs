/// E2Eテスト（Linuxコンテナ環境）
/// 
/// Git Worktreeラッパーとしての完全な動作をテストします。
/// すべてのテストはDockerコンテナ内で実行され、ローカル環境を汚染しません。

mod common;

use common::{TestRepo, TestEnvironment};

// =============================================================================
// 基本的なワークフロー
// =============================================================================

#[test]
fn test_full_workflow() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();
    
    // 1. addコマンドでworktree作成
    let worktree_path = repo.worktree_path("feature-work");
    let output = repo.run_twin(&["add", &worktree_path, "-b", "feature-branch"]);
    assert!(output.status.success(), "Failed to add worktree: {:?}", 
            String::from_utf8_lossy(&output.stderr));
    
    // 2. listコマンドで確認
    let output = repo.run_twin(&["list"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("feature-work") || stdout.contains("feature-branch"));
    
    // 3. removeコマンドで削除
    let output = repo.run_twin(&["remove", &worktree_path, "--force"]);
    assert!(output.status.success(), "Failed to remove worktree");
    
    // 4. 削除されたことを確認
    let output = repo.run_twin(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("feature-work"));
}

// =============================================================================
// git worktreeオプションのテスト
// =============================================================================

#[test]
fn test_worktree_options() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();
    
    // --detachオプションのテスト
    let output = repo.exec(&["git", "rev-parse", "HEAD"]);
    let _head_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    let detached_path = repo.worktree_path("detached");
    let output = repo.run_twin(&["add", &detached_path, "--detach", "HEAD"]);
    assert!(output.status.success(), "Detach mode should work: {:?}",
            String::from_utf8_lossy(&output.stderr));
    
    // --no-checkoutオプションのテスト
    let no_checkout_path = repo.worktree_path("no-checkout");
    let output = repo.run_twin(&["add", &no_checkout_path, "-b", "empty-branch", "--no-checkout"]);
    assert!(output.status.success(), "No-checkout should work: {:?}",
            String::from_utf8_lossy(&output.stderr));
    
    // --quietオプションのテスト
    let quiet_path = repo.worktree_path("quiet");
    let output = repo.run_twin(&["add", &quiet_path, "-b", "quiet-branch", "--quiet"]);
    assert!(output.status.success(), "Quiet mode failed: {:?}",
            String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty() || stdout.lines().count() <= 1);
}

// =============================================================================
// --git-onlyモードのテスト
// =============================================================================

#[test]
fn test_git_only_mode() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();
    
    // 設定ファイルを作成
    let config_content = r#"
[settings]
files = [
    { path = "test.txt" }
]

[hooks]
post_create = [
    { command = "echo", args = ["Hook executed"] }
]
"#;
    
    // 設定ファイルを作成
    repo.exec(&["sh", "-c", &format!("echo '{}' > .twin.toml", config_content)]);
    
    // --git-onlyモードで実行
    let worktree_path = repo.worktree_path("git-only");
    let output = repo.run_twin(&[
        "add", &worktree_path, "-b", "git-only-branch",
        "--config", ".twin.toml", "--git-only"
    ]);
    
    assert!(output.status.success(), "Failed to run twin add --git-only: {:?}",
            String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // フックが実行されていないことを確認
    assert!(!stdout.contains("Hook executed"));
}

// =============================================================================
// エラーハンドリングのテスト
// =============================================================================

#[test]
fn test_error_handling() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();
    
    // 存在しないブランチを指定
    let worktree_path = repo.worktree_path("error-test");
    let output = repo.run_twin(&["add", &worktree_path, "non-existent-branch"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("fatal:") || stderr.contains("not a valid"));
    
    // 無効なブランチ名
    let invalid_path = repo.worktree_path("invalid");
    let output = repo.run_twin(&["add", &invalid_path, "-b", "..invalid.."]);
    assert!(!output.status.success());
    
    // 既存のパスに作成
    repo.exec(&["mkdir", "../existing"]);
    repo.exec(&["touch", "../existing/file.txt"]);
    let output = repo.run_twin(&["add", "../existing", "-b", "test"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("not empty"));
}

// =============================================================================
// パス処理のテスト
// =============================================================================

#[test]
fn test_path_handling() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();
    
    // 相対パス
    let relative_path = repo.worktree_path("relative");
    let output = repo.run_twin(&["add", &relative_path, "-b", "path-test1"]);
    assert!(output.status.success(), "Relative path test failed: {:?}",
            String::from_utf8_lossy(&output.stderr));
    
    // 絶対パス
    let abs_path = format!("/tmp/absolute-{}", repo.test_id);
    let output = repo.run_twin(&["add", &abs_path, "-b", "path-test2"]);
    assert!(output.status.success(), "Absolute path test failed: {:?}",
            String::from_utf8_lossy(&output.stderr));
}

// =============================================================================
// 既存worktreeとの互換性
// =============================================================================

#[test]
fn test_compatibility_with_git_worktree() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::linux();
    
    // git worktreeで直接作成
    let compat_path = repo.worktree_path("compat");
    let output = repo.exec(&[
        "git", "worktree", "add", &compat_path, "-b", "compat-branch"
    ]);
    assert!(output.status.success());
    
    // twin listで表示されることを確認
    let output = repo.run_twin(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("compat") || stdout.contains("compat-branch"));
    
    // twin removeで削除できることを確認
    let output = repo.run_twin(&["remove", &compat_path, "--force"]);
    assert!(output.status.success());
    
    // 削除されたことを確認
    let output = repo.exec(&["git", "worktree", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("compat"));
}
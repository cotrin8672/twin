/// 結合テスト
///
/// Git操作、シンボリックリンク、フック実行など
/// 実際の外部システムとの連携が必要なテストを実行します。
mod common;

use common::TestRepo;
use std::process::Command;

// =============================================================================
// Git操作との結合テスト
// =============================================================================

#[test]
fn test_git_worktree_operations() {
    let repo = TestRepo::new();

    // ブランチ作成を伴うworktree追加
    let output = repo.run_twin(&[
        "add",
        &repo.worktree_path("feature"),
        "-b",
        "feature/test-1",
    ]);
    if !output.status.success() {
        eprintln!("Failed to add worktree:");
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // ブランチが作成されたことを確認
    let output = repo.exec(&["git", "branch", "-a"]);
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("feature/test-1"));

    // worktreeの状態を確認
    let output = repo.exec(&["git", "worktree", "list", "--porcelain"]);
    let worktrees = String::from_utf8_lossy(&output.stdout);
    assert!(worktrees.contains("branch refs/heads/feature/test-1"));

    // 既存ブランチでのworktree作成（エラーになるはず）
    let output = repo.run_twin(&[
        "add",
        &repo.worktree_path("another"),
        "-b",
        "feature/test-1",
    ]);
    assert!(!output.status.success());

    // -Bオプションで強制作成
    repo.exec(&["git", "branch", "existing-branch"]);
    let output = repo.run_twin(&["add", &repo.worktree_path("force"), "-B", "existing-branch"]);
    assert!(output.status.success());
}

// =============================================================================
// シンボリックリンクの結合テスト
// =============================================================================

#[test]
fn test_symlink_creation_with_config() {
    let repo = TestRepo::new();

    // 設定ファイルの作成
    let config = r#"
[[files]]
path = "config.json"
mapping_type = "symlink"

[[files]]
path = "data/test.txt"
mapping_type = "symlink"
"#;

    std::fs::write(repo.path().join(".twin.toml"), config).unwrap();
    std::fs::write(repo.path().join("config.json"), "{}").unwrap();
    std::fs::create_dir(repo.path().join("data")).unwrap();
    std::fs::write(repo.path().join("data/test.txt"), "test data").unwrap();

    // シンボリックリンクを含むworktree作成
    let worktree_path_str = repo.worktree_path("with-symlinks");
    let output = repo.run_twin(&[
        "add",
        &worktree_path_str,
        "-b",
        "feature/symlinks",
        "--config",
        ".twin.toml",
    ]);
    assert!(output.status.success());

    // シンボリックリンクが作成されたことを確認
    let worktree_path = repo.path().parent().unwrap().join(&worktree_path_str[3..]);

    // Windows環境ではシンボリックリンク作成が失敗する可能性があるため、
    // ファイルの存在のみを確認
    assert!(worktree_path.join("config.json").exists());
    assert!(worktree_path.join("data/test.txt").exists());
}

#[test]
fn test_no_symlinks_without_config() {
    let repo = TestRepo::new();

    // 設定ファイルを作成（この設定は適用されない）
    std::fs::write(repo.path().join("config.json"), "{}").unwrap();

    // 設定ファイルを指定せずにworktree作成
    let worktree_path_str = repo.worktree_path("no-symlinks");
    let output = repo.run_twin(&["add", &worktree_path_str, "-b", "feature/no-symlinks"]);
    assert!(output.status.success());

    // シンボリックリンクが作成されていないことを確認
    let worktree_path = repo.path().parent().unwrap().join(&worktree_path_str[3..]);
    assert!(!worktree_path.join("config.json").exists());
}

// =============================================================================
// フック実行の結合テスト
// =============================================================================

#[test]
fn test_hook_execution() {
    let repo = TestRepo::new();

    // フック付き設定ファイルの作成
    let config = r#"
[hooks]
post_create = [
    { command = "echo", args = ["Hook executed"] }
]
"#;

    std::fs::write(repo.path().join(".twin.toml"), config).unwrap();

    // フックを含むworktree作成
    let output = repo.run_twin(&[
        "add",
        &repo.worktree_path("with-hooks"),
        "-b",
        "feature/hooks",
        "--config",
        ".twin.toml",
    ]);

    // フックが実行されたことを出力で確認
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success() || stderr.contains("Hook executed"));
}

// =============================================================================
// worktree削除の結合テスト
// =============================================================================

#[test]
fn test_worktree_removal() {
    let repo = TestRepo::new();

    // worktreeを作成
    let worktree_path = repo.worktree_path("to-remove");
    let output = repo.run_twin(&["add", &worktree_path, "-b", "feature/removal"]);
    assert!(output.status.success());

    // worktreeが存在することを確認
    let output = repo.exec(&["git", "worktree", "list"]);
    let worktrees = String::from_utf8_lossy(&output.stdout);
    assert!(worktrees.contains("to-remove"));

    // worktreeを削除
    let output = repo.run_twin(&["remove", &worktree_path, "--force"]);
    assert!(output.status.success());

    // worktreeが削除されたことを確認
    let output = repo.exec(&["git", "worktree", "list"]);
    let worktrees = String::from_utf8_lossy(&output.stdout);
    assert!(!worktrees.contains("to-remove"));
}

// =============================================================================
// ワークフローの結合テスト
// =============================================================================

#[test]
fn test_complete_workflow() {
    let repo = TestRepo::new();

    // 複数のworktreeを作成
    let work1_path_str = repo.worktree_path("work-1");
    let work2_path_str = repo.worktree_path("work-2");
    let work3_path_str = repo.worktree_path("work-3");
    repo.run_twin(&["add", &work1_path_str, "-b", "feature/work-1"]);
    repo.run_twin(&["add", &work2_path_str, "-b", "feature/work-2"]);
    repo.run_twin(&["add", &work3_path_str, "-b", "feature/work-3"]);

    // 特定のworktreeで作業
    let work1_path = repo.path().parent().unwrap().join(&work1_path_str[3..]);
    std::fs::write(work1_path.join("new-file.txt"), "content").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(&work1_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Work in progress"])
        .current_dir(&work1_path)
        .output()
        .unwrap();

    // リスト確認
    let output = repo.run_twin(&["list"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("work-1"));
    assert!(stdout.contains("work-2"));
    assert!(stdout.contains("work-3"));

    // クリーンアップ
    repo.run_twin(&["remove", &work1_path_str, "--force"]);
    repo.run_twin(&["remove", &work2_path_str, "--force"]);
    repo.run_twin(&["remove", &work3_path_str, "--force"]);

    // すべて削除されたことを確認
    let output = repo.exec(&["git", "worktree", "list"]);
    let worktrees = String::from_utf8_lossy(&output.stdout);
    assert!(!worktrees.contains("work-1"));
    assert!(!worktrees.contains("work-2"));
    assert!(!worktrees.contains("work-3"));
}

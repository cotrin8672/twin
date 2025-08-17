/// Git Worktreeラッパーとしての機能をテストするモジュール
///
/// このテストモジュールは、twinがgit worktreeの純粋なラッパーとして
/// 正しく動作することを確認します。
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// テスト用のGitリポジトリをセットアップ
fn setup_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();

    // Gitリポジトリを初期化（デフォルトブランチ名を明示的に指定）
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");

    // Git設定（ローカルリポジトリのみ）
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to set git user email");

    // 初期コミットを作成
    fs::write(dir.path().join("README.md"), "# Test Repo").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output()
        .expect("Failed to add files");
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to commit");

    dir
}

/// twinバイナリのパスを取得
fn get_twin_binary() -> String {
    let exe_path = std::env::current_exe().unwrap();
    let target_dir = exe_path.parent().unwrap().parent().unwrap();
    let twin_path = target_dir.join("twin");

    twin_path.to_string_lossy().to_string()
}

/// ユニークなワークツリーパスを生成（リポジトリ内に作成）
fn unique_worktree_path(name: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string()[0..8].to_string();
    format!("wt-{name}-{id}")
}

// =============================================================================
// 1. addコマンドの基本テスト
// =============================================================================

#[test]
fn test_add_command_basic() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("add");

    // twin add コマンドを実行
    let output = Command::new(&twin)
        .args(["add", &worktree_path, "-b", "test-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    eprintln!("STDOUT: {stdout}");
    eprintln!("STDERR: {stderr}");

    assert!(
        output.status.success(),
        "twin add should succeed. stderr: {stderr}"
    );

    // worktreeが作成されたことを確認
    let list_output = Command::new("git")
        .args(["worktree", "list"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to list worktrees");

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains(&worktree_path));
    assert!(list_stdout.contains("test-branch"));
}

#[test]
fn test_add_without_branch_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("nobranch");

    // ブランチオプションなしでaddを実行
    // git worktreeと同様、HEADの状態でworktreeを作成する
    let output = Command::new(&twin)
        .args(["add", &worktree_path])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    // git worktreeと同様の動作：ブランチ指定がない場合もworktreeは作成される
    // （detached HEADまたはHEADのブランチを使用）
    assert!(
        output.status.success(),
        "twin add without branch should succeed like git worktree: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_force_branch_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path1 = unique_worktree_path("force1");
    let worktree_path2 = unique_worktree_path("force2");

    // 既存のブランチを作成
    Command::new("git")
        .args(["branch", "existing-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to create branch");

    // 最初のworktreeを作成
    Command::new(&twin)
        .args(["add", &worktree_path1, "-b", "existing-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    // -Bオプションで同じブランチ名を強制作成
    let output = Command::new(&twin)
        .args(["add", &worktree_path2, "-B", "existing-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    assert!(output.status.success(), "twin add with -B should succeed");
}

#[test]
fn test_detach_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("detached");

    // HEADのコミットハッシュを取得
    let head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to get HEAD");
    let _head_commit = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    // --detachオプションでworktreeを作成
    let output = Command::new(&twin)
        .args(["add", &worktree_path, "--detach"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    assert!(
        output.status.success(),
        "Detach should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // worktreeがdetached状態であることを確認
    let list_output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to list worktrees");

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("detached"));
}

#[test]
fn test_lock_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("locked");

    // --lockオプションでworktreeを作成
    let output = Command::new(&twin)
        .args(["add", &worktree_path, "-b", "locked-branch", "--lock"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    assert!(output.status.success());

    // worktreeがロックされていることを確認
    let list_output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to list worktrees");

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("locked"));
}

#[test]
fn test_no_checkout_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("nocheckout");

    // --no-checkoutオプションでworktreeを作成
    // twin add <branch> [path] のフォーマット
    let output = Command::new(&twin)
        .args([
            "add",
            "empty-branch",
            &worktree_path,
            "-b",
            "empty-branch",
            "--no-checkout",
        ])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("STDOUT: {stdout}");
    println!("STDERR: {stderr}");

    assert!(
        output.status.success(),
        "No-checkout should succeed: {stderr}",
    );

    // worktreeディレクトリが空であることを確認
    let worktree_full_path = repo.path().join(&worktree_path);
    let entries: Vec<_> = fs::read_dir(&worktree_full_path)
        .expect("Failed to read worktree dir")
        .collect();
    // .gitディレクトリのみが存在するはず
    assert!(entries.len() <= 1, "Worktree should be nearly empty");
}

#[test]
fn test_quiet_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("quiet");

    // --quietオプションでworktreeを作成
    let output = Command::new(&twin)
        .args(["add", &worktree_path, "-b", "quiet-branch", "--quiet"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    assert!(output.status.success());

    // 出力が抑制されていることを確認
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "Quiet mode should suppress output"
    );
}

#[test]
fn test_git_only_mode() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("gitonly");

    // 設定ファイルを作成
    let config = r#"
[[files]]
path = "test.txt"
mapping_type = "symlink"
"#;
    fs::write(repo.path().join(".twin.toml"), config).unwrap();
    fs::write(repo.path().join("test.txt"), "test content").unwrap();

    // --git-onlyオプションでworktreeを作成
    let output = Command::new(&twin)
        .args([
            "add",
            &worktree_path,
            "-b",
            "git-only-branch",
            "--config",
            ".twin.toml",
            "--git-only",
        ])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    assert!(output.status.success());

    // シンボリックリンクが作成されていないことを確認
    let worktree_full_path = repo.path().join(&worktree_path);
    assert!(!worktree_full_path.join("test.txt").exists());
}

// =============================================================================
// 2. エラーハンドリングのテスト
// =============================================================================

#[test]
fn test_error_message_passthrough() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // 既に存在するブランチで同じパスにworktreeを作成しようとする（エラーになる）
    let worktree_path = unique_worktree_path("error");

    // 最初のworktreeを作成
    Command::new(&twin)
        .args(["add", &worktree_path, "-b", "test-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute first twin add");

    // 同じパスで別のworktreeを作成しようとする
    let output = Command::new(&twin)
        .args(["add", &worktree_path, "-b", "another-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    assert!(!output.status.success());

    // gitのエラーメッセージが表示されることを確認
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("fatal") || stderr.contains("already exists")
    );
}

#[test]
fn test_invalid_branch_name_error() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("invalid");

    // 無効なブランチ名でaddを実行
    let output = Command::new(&twin)
        .args(["add", &worktree_path, "-b", "invalid..branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("fatal"),
        "Should show branch name error"
    );
}

// =============================================================================
// 3. listコマンドのテスト
// =============================================================================

#[test]
fn test_list_includes_manual_worktree() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("manual");

    // 手動でgit worktreeを作成
    Command::new("git")
        .args(["worktree", "add", &worktree_path, "-b", "manual-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to create manual worktree");

    // twin listを実行
    let output = Command::new(&twin)
        .args(["list"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin list");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("manual-branch"));
}

// =============================================================================
// 4. removeコマンドのテスト
// =============================================================================

#[test]
fn test_remove_manual_worktree() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path = unique_worktree_path("remove");

    // 手動でgit worktreeを作成
    Command::new("git")
        .args(["worktree", "add", &worktree_path, "-b", "to-remove"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to create manual worktree");

    // twin removeを実行
    let output = Command::new(&twin)
        .args(["remove", &worktree_path, "--force"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin remove");

    assert!(output.status.success());

    // worktreeが削除されたことを確認
    let list_output = Command::new("git")
        .args(["worktree", "list"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to list worktrees");

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(!list_stdout.contains("to-remove"));
}

// =============================================================================
// 5. git worktreeとの一致性テスト
// =============================================================================

#[test]
fn test_output_matches_git_worktree() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();
    let worktree_path1 = unique_worktree_path("git1");
    let worktree_path2 = unique_worktree_path("twin1");

    // git worktree addを直接実行
    let git_output = Command::new("git")
        .args(["worktree", "add", &worktree_path1, "-b", "git-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute git worktree add");

    // twin addを実行
    let twin_output = Command::new(&twin)
        .args(["add", &worktree_path2, "-b", "twin-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to execute twin add");

    // 両方とも成功することを確認
    assert_eq!(git_output.status.success(), twin_output.status.success());

    // worktree listで両方が表示されることを確認
    let list_output = Command::new("git")
        .args(["worktree", "list"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to list worktrees");

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("git-branch"));
    assert!(list_stdout.contains("twin-branch"));
}

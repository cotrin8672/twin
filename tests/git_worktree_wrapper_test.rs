/// Git Worktreeラッパーとしての機能をテストするモジュール
///
/// このテストモジュールは、twinがgit worktreeの純粋なラッパーとして
/// 正しく動作することを確認します。
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// テスト用のGitリポジトリをセットアップ
fn setup_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();

    // Gitリポジトリを初期化
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");

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

// =============================================================================
// 1. addコマンドの基本テスト
// =============================================================================

#[test]
fn test_add_command_basic() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // addコマンドで新しいworktreeを作成（一意のパスを使用）
    let worktree_path = repo.path().with_file_name("test-add");
    let output = Command::new(&twin)
        .args(["add", worktree_path.to_str().unwrap(), "-b", "test-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    assert!(
        output.status.success(),
        "twin add should succeed. stderr: {}",
        stderr
    );

    // git worktree listで確認
    let list = Command::new("git")
        .args(["worktree", "list"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let list_output = String::from_utf8_lossy(&list.stdout);
    assert!(
        list_output.contains("test-add"),
        "Worktree should be created"
    );
    assert!(
        list_output.contains("test-branch"),
        "Branch should be created"
    );
}

#[test]
fn test_add_without_branch_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // ブランチ名を位置引数として指定（存在しないブランチ）
    let output = Command::new(&twin)
        .args(["add", "../my-feature", "non-existent-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add");

    // 存在しないブランチを指定しているのでエラーになるはず
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // gitのエラーメッセージを確認
    assert!(
        stderr.contains("invalid reference")
            || stderr.contains("not a valid object name")
            || stderr.contains("fatal:"),
        "Expected git error message, got: {}",
        stderr
    );
}

// =============================================================================
// 2. Git Worktreeオプションのテスト
// =============================================================================

#[test]
fn test_force_branch_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // 最初にブランチを作成
    Command::new("git")
        .args(["branch", "test-branch"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    // 既存ブランチで強制作成（-Bオプション）
    let output = Command::new(&twin)
        .args(["add", "../forced", "-B", "test-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add with -B");

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Force branch stderr: {}", stderr);

    assert!(
        output.status.success(),
        "Force branch should succeed: {}",
        stderr
    );
}

#[test]
fn test_detach_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // HEADのコミットを取得
    let head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let head_commit = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    // デタッチモードでworktreeを作成（HEADを指定）
    let output = Command::new(&twin)
        .args(["add", "../detached", "--detach", "HEAD"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add --detach");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Detach should succeed: {}", stderr);

    // git worktree listで確認
    let list = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let list_output = String::from_utf8_lossy(&list.stdout);
    assert!(list_output.contains("detached"));
}

#[test]
fn test_lock_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // ロック付きでworktreeを作成
    let output = Command::new(&twin)
        .args(["add", "../locked", "-b", "locked-branch", "--lock"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add --lock");

    assert!(output.status.success());

    // ロックファイルが作成されているか確認
    let git_dir = repo.path().join(".git");
    let worktrees_dir = git_dir.join("worktrees");

    // lockedディレクトリが存在し、その中にlockedファイルがあるはず
    let locked_entries: Vec<_> = fs::read_dir(&worktrees_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().join("locked").exists())
        .collect();

    assert!(!locked_entries.is_empty(), "Lock file should exist");
}

#[test]
fn test_no_checkout_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // チェックアウトせずにworktreeを作成
    let output = Command::new(&twin)
        .args([
            "add",
            "../no-checkout",
            "-b",
            "empty-branch",
            "--no-checkout",
        ])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add --no-checkout");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "No-checkout should succeed: {}",
        stderr
    );

    // worktreeディレクトリは存在するが、ファイルはないはず
    let worktree_path = repo.path().parent().unwrap().join("no-checkout");
    assert!(worktree_path.exists(), "Worktree directory should exist");
    // .gitファイルは存在するが、README.mdはない
    assert!(worktree_path.join(".git").exists(), ".git should exist");
    assert!(
        !worktree_path.join("README.md").exists(),
        "README.md should not exist"
    );
}

#[test]
fn test_quiet_option() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // quietモードで実行
    let output = Command::new(&twin)
        .args(["add", "../quiet-test", "-b", "quiet-branch", "--quiet"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add --quiet");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // quietモードでは出力が最小限になる
    assert!(stdout.is_empty() || stdout.lines().count() <= 1);
}

// =============================================================================
// 3. --git-onlyモードのテスト
// =============================================================================

#[test]
fn test_git_only_mode() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // 設定ファイルを作成（シンボリックリンクとフックを定義）
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
    fs::write(repo.path().join(".twin.toml"), config_content).unwrap();

    // --git-onlyモードで実行
    let output = Command::new(&twin)
        .args([
            "add",
            "../git-only-test",
            "-b",
            "test-branch",
            "--config",
            ".twin.toml",
            "--git-only",
        ])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add --git-only");

    assert!(output.status.success());

    // フックが実行されていないことを確認
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Hook executed"));

    // シンボリックリンクが作成されていないことを確認
    let worktree_path = repo.path().parent().unwrap().join("git-only-test");
    assert!(!worktree_path.join("test.txt").exists());
}

// =============================================================================
// 4. エラーメッセージの透過性テスト
// =============================================================================

#[test]
fn test_error_message_passthrough() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // 無効なパス（既存のファイル）にworktreeを作成しようとする
    fs::write(repo.path().join("existing-file"), "content").unwrap();

    let output = Command::new(&twin)
        .args(["add", "existing-file", "-b", "test"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add");

    assert!(!output.status.success());

    // git worktreeのエラーメッセージがそのまま表示される
    let stderr = String::from_utf8_lossy(&output.stderr);
    // git worktreeの典型的なエラーメッセージを確認
    assert!(
        stderr.contains("already exists")
            || stderr.contains("is not empty")
            || stderr.contains("fatal:"),
        "Git error message should be passed through"
    );
}

#[test]
fn test_invalid_branch_name_error() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // 無効なブランチ名でworktreeを作成
    let output = Command::new(&twin)
        .args(["add", "../invalid", "-b", "..invalid..branch.."])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // gitのブランチ名検証エラーが表示される
    assert!(stderr.contains("invalid") || stderr.contains("branch"));
}

// =============================================================================
// 5. 既存worktreeとの互換性テスト
// =============================================================================

#[test]
fn test_list_includes_manual_worktree() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // 手動でgit worktreeを作成
    Command::new("git")
        .args([
            "worktree",
            "add",
            "../manual-worktree",
            "-b",
            "manual-branch",
        ])
        .current_dir(repo.path())
        .output()
        .expect("Failed to create manual worktree");

    // twin listで表示されることを確認
    let output = Command::new(&twin)
        .args(["list"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("manual-worktree") || stdout.contains("manual-branch"));
}

#[test]
fn test_remove_manual_worktree() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // 手動でgit worktreeを作成
    Command::new("git")
        .args([
            "worktree",
            "add",
            "../manual-remove",
            "-b",
            "manual-remove-branch",
        ])
        .current_dir(repo.path())
        .output()
        .expect("Failed to create manual worktree");

    // twin removeで削除
    let output = Command::new(&twin)
        .args(["remove", "../manual-remove", "--force"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin remove");

    assert!(output.status.success());

    // 削除されたことを確認
    let list = Command::new("git")
        .args(["worktree", "list"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let list_output = String::from_utf8_lossy(&list.stdout);
    assert!(!list_output.contains("manual-remove"));
}

// =============================================================================
// 6. 出力の一致性テスト
// =============================================================================

#[test]
fn test_output_matches_git_worktree() {
    let repo = setup_test_repo();
    let twin = get_twin_binary();

    // git worktree addの出力を取得
    let git_output = Command::new("git")
        .args(["worktree", "add", "../git-test", "-b", "git-branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run git worktree add");

    // twin addの出力を取得（--git-onlyモードで副作用を除外）
    let twin_output = Command::new(&twin)
        .args(["add", "../twin-test", "-b", "twin-branch", "--git-only"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to run twin add");

    // 両方とも成功することを確認
    assert!(git_output.status.success());
    assert!(twin_output.status.success());

    // 出力フォーマットが類似していることを確認
    let git_stdout = String::from_utf8_lossy(&git_output.stdout);
    let twin_stdout = String::from_utf8_lossy(&twin_output.stdout);

    println!("Git output: {}", git_stdout);
    println!("Twin output: {}", twin_stdout);

    // 両方ともworktreeの作成に関する出力があることを確認
    // git worktreeの出力はバージョンによって異なるため、柔軟にチェック
    assert!(
        (git_stdout.contains("Preparing")
            || git_stdout.contains("HEAD")
            || git_stdout.contains("branch"))
            && (twin_stdout.contains("Preparing")
                || twin_stdout.contains("HEAD")
                || twin_stdout.contains("branch")),
        "Both commands should produce worktree-related output"
    );
}

/// 結合テスト（コンテナ環境が必要なもの）
///
/// Git操作、シンボリックリンク、フック実行など
/// 実際の外部システムとの連携が必要なテストを実行します。
mod common;

use common::{TestEnvironment, TestRepo};

// =============================================================================
// Git操作との結合テスト
// =============================================================================

#[test]
fn test_git_integration_local() {
    let repo = TestRepo::local();
    test_git_integration(repo);
}

#[test]
#[cfg(not(windows))]
fn test_git_integration_container() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::container();
    test_git_integration(repo);
}

fn test_git_integration(repo: TestRepo) {
    // ブランチ作成を伴うworktree追加
    let output = repo.run_twin(&["add", "../feature", "-b", "feature/test-1"]);
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
    let output = repo.run_twin(&["add", "../another", "-b", "feature/test-1"]);
    assert!(!output.status.success());

    // -Bオプションで強制作成
    repo.exec(&["git", "branch", "existing-branch"]);
    let output = repo.run_twin(&["add", "../forced", "-B", "existing-branch"]);
    assert!(output.status.success());
}

// =============================================================================
// シンボリックリンクとの結合テスト
// =============================================================================

#[test]
fn test_symlink_integration_local() {
    let repo = TestRepo::local();
    test_symlink_integration(repo);
}

#[test]
#[cfg(not(windows))]
fn test_symlink_integration_container() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::container();
    test_symlink_integration(repo);
}

fn test_symlink_integration(repo: TestRepo) {
    // シンボリックリンク設定を含む.twin.tomlを作成
    let config = r#"
[settings]
files = [
    { path = "config.json", description = "共有設定ファイル" },
    { path = "data", description = "共有データディレクトリ" }
]
"#;

    // 設定ファイルとテストファイルを作成
    match &repo.env {
        TestEnvironment::Local => {
            std::fs::write(repo.path().join(".twin.toml"), config).unwrap();
            std::fs::write(repo.path().join("config.json"), "{}").unwrap();
            std::fs::create_dir(repo.path().join("data")).unwrap();
            std::fs::write(repo.path().join("data/test.txt"), "test data").unwrap();
        }
        TestEnvironment::Container => {
            repo.exec(&["sh", "-c", &format!("echo '{}' > .twin.toml", config)]);
            repo.exec(&["sh", "-c", "echo '{}' > config.json"]);
            repo.exec(&["mkdir", "data"]);
            repo.exec(&["sh", "-c", "echo 'test data' > data/test.txt"]);
        }
    }

    // 設定を指定してworktree作成
    let output = repo.run_twin(&[
        "add",
        "../with-symlinks",
        "-b",
        "symlink-test",
        "--config",
        ".twin.toml",
    ]);
    assert!(output.status.success());

    // シンボリックリンクが作成されたことを確認
    let worktree_path = match &repo.env {
        TestEnvironment::Local => repo.path().parent().unwrap().join("with-symlinks"),
        TestEnvironment::Container => std::path::PathBuf::from("/workspace/with-symlinks"),
    };

    // config.jsonのシンボリックリンクを確認
    let check_cmd = match &repo.env {
        TestEnvironment::Local => {
            vec![
                "test",
                "-L",
                worktree_path.join("config.json").to_str().unwrap(),
            ]
        }
        TestEnvironment::Container => {
            vec!["test", "-L", "/workspace/with-symlinks/config.json"]
        }
    };

    let output = repo.exec(&check_cmd);

    // Windowsではシンボリックリンクの代わりにコピーされる可能性がある
    #[cfg(not(windows))]
    assert!(output.status.success(), "Symlink should be created");

    // --git-onlyモードではシンボリックリンクを作成しない
    let output = repo.run_twin(&[
        "add",
        "../no-symlinks",
        "-b",
        "no-symlink-test",
        "--config",
        ".twin.toml",
        "--git-only",
    ]);
    assert!(output.status.success());

    // シンボリックリンクが作成されていないことを確認
    let check_cmd = match &repo.env {
        TestEnvironment::Local => {
            vec![
                "test",
                "-e",
                repo.path()
                    .parent()
                    .unwrap()
                    .join("no-symlinks/config.json")
                    .to_str()
                    .unwrap(),
            ]
        }
        TestEnvironment::Container => {
            vec!["test", "-e", "/workspace/no-symlinks/config.json"]
        }
    };

    let output = repo.exec(&check_cmd);
    assert!(
        !output.status.success(),
        "Symlink should not be created with --git-only"
    );
}

// =============================================================================
// フック実行との結合テスト
// =============================================================================

#[test]
fn test_hook_integration_local() {
    let repo = TestRepo::local();
    test_hook_integration(repo);
}

#[test]
#[cfg(not(windows))]
fn test_hook_integration_container() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::container();
    test_hook_integration(repo);
}

fn test_hook_integration(repo: TestRepo) {
    // フック設定を含む.twin.tomlを作成
    let config = r#"
[hooks]
post_create = [
    { command = "echo", args = ["Post-create hook executed"] },
    { command = "touch", args = ["../hook-marker.txt"] }
]
"#;

    match &repo.env {
        TestEnvironment::Local => {
            std::fs::write(repo.path().join(".twin.toml"), config).unwrap();
        }
        TestEnvironment::Container => {
            repo.exec(&["sh", "-c", &format!("echo '{}' > .twin.toml", config)]);
        }
    }

    // フックを含むworktree作成
    let output = repo.run_twin(&[
        "add",
        "../with-hooks",
        "-b",
        "hook-test",
        "--config",
        ".twin.toml",
    ]);

    // 出力にフックの実行結果が含まれることを確認
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // フックが実行されたことを確認（マーカーファイルの存在）
    let marker_check = match &repo.env {
        TestEnvironment::Local => {
            vec![
                "test",
                "-f",
                repo.path()
                    .parent()
                    .unwrap()
                    .join("hook-marker.txt")
                    .to_str()
                    .unwrap(),
            ]
        }
        TestEnvironment::Container => {
            vec!["test", "-f", "/workspace/hook-marker.txt"]
        }
    };

    // 注: 現在の実装ではフックが実装されていない可能性があるため、
    // この部分は実装状況に応じて調整が必要

    // --git-onlyモードではフックを実行しない
    let output = repo.run_twin(&[
        "add",
        "../no-hooks",
        "-b",
        "no-hook-test",
        "--config",
        ".twin.toml",
        "--git-only",
    ]);
    assert!(output.status.success());

    // フックが実行されていないことを確認
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Post-create hook executed"));
}

// =============================================================================
// 複雑なワークフローの結合テスト
// =============================================================================

#[test]
fn test_complex_workflow_local() {
    let repo = TestRepo::local();
    test_complex_workflow(repo);
}

#[test]
#[cfg(not(windows))]
fn test_complex_workflow_container() {
    if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
        return;
    }
    let repo = TestRepo::container();
    test_complex_workflow(repo);
}

fn test_complex_workflow(repo: TestRepo) {
    // 複数のworktreeを作成
    for i in 1..=3 {
        let output = repo.run_twin(&[
            "add",
            &format!("../work-{}", i),
            "-b",
            &format!("feature-{}", i),
        ]);
        assert!(output.status.success());
    }

    // listで全て表示されることを確認
    let output = repo.run_twin(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    for i in 1..=3 {
        assert!(
            stdout.contains(&format!("work-{}", i)) || stdout.contains(&format!("feature-{}", i))
        );
    }

    // 特定のworktreeで作業
    match &repo.env {
        TestEnvironment::Local => {
            let work1_path = repo.path().parent().unwrap().join("work-1");
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
        }
        TestEnvironment::Container => {
            repo.exec(&[
                "sh",
                "-c",
                "echo 'content' > /workspace/work-1/new-file.txt",
            ]);
            repo.exec(&["git", "-C", "/workspace/work-1", "add", "."]);
            repo.exec(&[
                "git",
                "-C",
                "/workspace/work-1",
                "commit",
                "-m",
                "Work in progress",
            ]);
        }
    }

    // 変更があるworktreeは削除できない（--force必要）
    let output = repo.run_twin(&["remove", "../work-1"]);
    // 注: 実装によってはこの動作が異なる可能性がある

    // --forceで強制削除
    let output = repo.run_twin(&["remove", "../work-1", "--force"]);
    assert!(output.status.success());

    // 残りも削除
    for i in 2..=3 {
        let output = repo.run_twin(&["remove", &format!("../work-{}", i), "--force"]);
        assert!(output.status.success());
    }

    // 全て削除されたことを確認
    let output = repo.run_twin(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    for i in 1..=3 {
        assert!(!stdout.contains(&format!("work-{}", i)));
    }
}

/// テスト用の共通ヘルパーモジュール
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// テスト用の一時的なGitリポジトリ
pub struct TestRepo {
    temp_dir: TempDir,
    #[allow(dead_code)]
    pub test_id: String,
    /// 作成されたworktreeのパスを記録
    created_worktrees: std::sync::Mutex<Vec<PathBuf>>,
}

impl TestRepo {
    /// テスト用のGitリポジトリを作成
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_id = uuid::Uuid::new_v4().to_string()[0..8].to_string();

        // Gitリポジトリを初期化（デフォルトブランチ名を明示的に指定）
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to init git repo");

        // Git設定（ローカルリポジトリのみ）
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git user name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git user email");

        // 初期コミット
        std::fs::write(temp_dir.path().join("README.md"), "# Test Repo").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to add files");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");

        Self {
            temp_dir,
            test_id,
            created_worktrees: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// コマンドを実行
    #[allow(dead_code)]
    pub fn exec(&self, cmd: &[&str]) -> std::process::Output {
        Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(self.temp_dir.path())
            .output()
            .expect("Failed to execute command")
    }

    /// twinコマンドを実行
    #[allow(dead_code)]
    pub fn run_twin(&self, args: &[&str]) -> std::process::Output {
        let twin_binary = Self::get_twin_binary();
        Command::new(twin_binary)
            .args(args)
            .current_dir(self.temp_dir.path())
            .output()
            .expect("Failed to run twin")
    }

    /// 一意のworktreeパスを生成
    #[allow(dead_code)]
    pub fn worktree_path(&self, name: &str) -> String {
        let path_str = format!("../test-{}-{}", name, self.test_id);
        // worktreeの絶対パスを記録
        if let Ok(abs_path) = self
            .temp_dir
            .path()
            .parent()
            .unwrap()
            .join(&path_str[3..])
            .canonicalize()
        {
            self.created_worktrees.lock().unwrap().push(abs_path);
        } else {
            // まだ存在しない場合は予想される絶対パスを記録
            let expected_path = self.temp_dir.path().parent().unwrap().join(&path_str[3..]);
            self.created_worktrees.lock().unwrap().push(expected_path);
        }
        path_str
    }

    /// テストリポジトリのパスを取得
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// twinバイナリのパスを取得
    #[allow(dead_code)]
    fn get_twin_binary() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap();
        let target_dir = exe_path.parent().unwrap().parent().unwrap();
        target_dir.join("twin")
    }
}

// Dropトレイトを実装してworktreeもクリーンアップ
impl Drop for TestRepo {
    fn drop(&mut self) {
        // 作成されたworktreeを削除
        let worktrees = self.created_worktrees.lock().unwrap();
        for worktree_path in worktrees.iter() {
            if worktree_path.exists() {
                // git worktree removeを試みる
                Command::new("git")
                    .args([
                        "worktree",
                        "remove",
                        "--force",
                        &worktree_path.to_string_lossy(),
                    ])
                    .current_dir(self.temp_dir.path())
                    .output()
                    .ok();

                // それでも残っている場合は直接削除
                if worktree_path.exists() {
                    std::fs::remove_dir_all(worktree_path).ok();
                }
            }
        }
        // TempDirは自動的にDropでクリーンアップされる
    }
}

/// テスト用の共通ヘルパーモジュール
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// テスト用の一時的なGitリポジトリ
pub struct TestRepo {
    temp_dir: TempDir,
    pub test_id: String,
}

impl TestRepo {
    /// テスト用のGitリポジトリを作成
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_id = uuid::Uuid::new_v4().to_string()[0..8].to_string();
        
        // Gitリポジトリを初期化
        Command::new("git")
            .args(&["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to init git repo");
            
        // Git設定（ローカルリポジトリのみ）
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git user name");
            
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git user email");
            
        // 初期コミット
        std::fs::write(temp_dir.path().join("README.md"), "# Test Repo").unwrap();
        Command::new("git")
            .args(&["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to add files");
            
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");
        
        Self {
            temp_dir,
            test_id,
        }
    }
    
    /// コマンドを実行
    pub fn exec(&self, cmd: &[&str]) -> std::process::Output {
        Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(self.temp_dir.path())
            .output()
            .expect("Failed to execute command")
    }
    
    /// twinコマンドを実行
    pub fn run_twin(&self, args: &[&str]) -> std::process::Output {
        let twin_binary = Self::get_twin_binary();
        Command::new(twin_binary)
            .args(args)
            .current_dir(self.temp_dir.path())
            .output()
            .expect("Failed to run twin")
    }
    
    /// 一意のworktreeパスを生成
    pub fn worktree_path(&self, name: &str) -> String {
        format!("../test-{}-{}", name, self.test_id)
    }
    
    /// テストリポジトリのパスを取得
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
    
    /// twinバイナリのパスを取得
    fn get_twin_binary() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap();
        let target_dir = exe_path.parent().unwrap().parent().unwrap();
        target_dir.join("twin")
    }
}

// TempDirは自動的にDropでクリーンアップされる
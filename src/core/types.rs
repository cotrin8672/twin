use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// エージェント環境の情報
/// 作成された環境の状態を保持し、永続化する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEnvironment {
    /// エージェント名（一意の識別子）
    pub name: String,
    
    /// Gitブランチ名
    pub branch: String,
    
    /// Worktreeのパス
    pub worktree_path: PathBuf,
    
    /// 作成されたシンボリックリンクのリスト
    pub symlinks: Vec<SymlinkInfo>,
    
    /// 環境の状態
    pub status: EnvironmentStatus,
    
    /// 作成日時
    pub created_at: DateTime<Utc>,
    
    /// 最終更新日時
    pub updated_at: DateTime<Utc>,
    
    /// 自動コミットが有効かどうか
    pub auto_commit_enabled: bool,
    
    /// 使用した設定ファイルのパス
    pub config_path: Option<PathBuf>,
}

/// 環境の状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvironmentStatus {
    /// アクティブ（現在使用中）
    Active,
    
    /// 非アクティブ（作成済みだが使用していない）
    Inactive,
    
    /// 作成中
    Creating,
    
    /// 削除中
    Removing,
    
    /// エラー状態
    Error(String),
}

/// シンボリックリンクの情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkInfo {
    /// リンク元のパス（絶対パス）
    pub source: PathBuf,
    
    /// リンク先のパス（絶対パス）
    pub target: PathBuf,
    
    /// リンクが正常に作成されたか
    pub is_valid: bool,
    
    /// エラーメッセージ（作成に失敗した場合）
    pub error_message: Option<String>,
}

impl AgentEnvironment {
    /// 新しい環境を作成
    pub fn new(
        name: String,
        branch: String,
        worktree_path: PathBuf,
        config_path: Option<PathBuf>,
    ) -> Self {
        let now = Utc::now();
        Self {
            name,
            branch,
            worktree_path,
            symlinks: Vec::new(),
            status: EnvironmentStatus::Creating,
            created_at: now,
            updated_at: now,
            auto_commit_enabled: false,
            config_path,
        }
    }
    
    /// 環境がアクティブかどうか
    pub fn is_active(&self) -> bool {
        self.status == EnvironmentStatus::Active
    }
    
    /// 環境のパスを取得
    pub fn path(&self) -> &PathBuf {
        &self.worktree_path
    }
    
    /// シンボリックリンクを追加
    pub fn add_symlink(&mut self, symlink: SymlinkInfo) {
        self.symlinks.push(symlink);
        self.updated_at = Utc::now();
    }
    
    /// 状態を更新
    pub fn set_status(&mut self, status: EnvironmentStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl SymlinkInfo {
    /// 新しいシンボリックリンク情報を作成
    pub fn new(source: PathBuf, target: PathBuf) -> Self {
        Self {
            source,
            target,
            is_valid: false,
            error_message: None,
        }
    }
    
    /// 成功状態として設定
    pub fn set_success(&mut self) {
        self.is_valid = true;
        self.error_message = None;
    }
    
    /// エラー状態として設定
    pub fn set_error(&mut self, message: String) {
        self.is_valid = false;
        self.error_message = Some(message);
    }
}
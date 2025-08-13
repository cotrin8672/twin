use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// グローバル設定とプロジェクト設定のマージ結果
    pub settings: ConfigSettings,

    /// 設定ファイルのパス（プロジェクト設定の場合）
    pub path: Option<PathBuf>,

    /// グローバル設定のパス（存在する場合）
    pub global_path: Option<PathBuf>,
}

/// 設定の実際の内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSettings {
    /// Git管理外ファイルの定義
    pub files: Vec<FileMapping>,

    /// フック設定
    pub hooks: HookConfig,

    /// Worktreeのベースディレクトリ
    pub worktree_base: Option<PathBuf>,

    /// デフォルトのブランチプレフィックス
    pub branch_prefix: Option<String>,
}

/// Git管理外ファイルのマッピング定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    /// ファイルパス（相対パス）
    pub path: PathBuf,

    /// マッピングタイプ（symlink or copy）
    #[serde(default = "default_mapping_type")]
    pub mapping_type: MappingType,

    /// 説明（オプション）
    pub description: Option<String>,

    /// 既に存在する場合はスキップ
    #[serde(default)]
    pub skip_if_exists: bool,
}

/// マッピングタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MappingType {
    /// シンボリックリンク（実体を共有）
    Symlink,
    /// ファイルコピー（各環境で独立）
    Copy,
}

fn default_mapping_type() -> MappingType {
    MappingType::Symlink
}

/// フック設定
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct HookConfig {
    /// 環境作成前のフック
    #[serde(default)]
    pub pre_create: Vec<HookCommand>,

    /// 環境作成後のフック
    #[serde(default)]
    pub post_create: Vec<HookCommand>,

    /// 環境削除前のフック
    #[serde(default)]
    pub pre_remove: Vec<HookCommand>,

    /// 環境削除後のフック
    #[serde(default)]
    pub post_remove: Vec<HookCommand>,
}

/// フックコマンドの定義
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookCommand {
    /// 実行するコマンド
    pub command: String,
    
    /// コマンド引数
    #[serde(default)]
    pub args: Vec<String>,
    
    /// 環境変数
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    /// タイムアウト（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// エラー時も処理を続行するか
    #[serde(default)]
    pub continue_on_error: bool,
}

fn default_timeout() -> u64 {
    60 // デフォルト60秒
}


/// 部分的失敗時の状態を管理する構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialFailureState {
    /// 操作のタイプ
    pub operation: OperationType,

    /// 成功したステップ
    pub succeeded_steps: Vec<OperationStep>,

    /// 失敗したステップ
    pub failed_step: Option<OperationStep>,

    /// ロールバック可能かどうか
    pub can_rollback: bool,

    /// エラーメッセージ
    pub error: Option<String>,
}

/// 操作のタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    CreateEnvironment,
    RemoveEnvironment,
    SwitchEnvironment,
    CreateSymlinks,
    RemoveSymlinks,
}

/// 操作のステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStep {
    /// ステップ名
    pub name: String,

    /// ステップの詳細
    pub details: HashMap<String, String>,

    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,

    /// ロールバック可能かどうか
    pub can_rollback: bool,
}

/// 環境レジストリ
/// すべての環境の状態を管理
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentRegistry {
    /// 環境のマップ（名前 -> 環境）
    pub environments: HashMap<String, AgentEnvironment>,

    /// アクティブな環境の名前
    pub active: Option<String>,

    /// 最終更新日時
    pub last_updated: Option<DateTime<Utc>>,
}

impl EnvironmentRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 環境を追加
    pub fn add(&mut self, env: AgentEnvironment) {
        self.environments.insert(env.name.clone(), env);
        self.last_updated = Some(Utc::now());
    }

    /// 環境を削除
    pub fn remove(&mut self, name: &str) -> Option<AgentEnvironment> {
        let env = self.environments.remove(name);
        if self.active.as_deref() == Some(name) {
            self.active = None;
        }
        self.last_updated = Some(Utc::now());
        env
    }

    /// アクティブな環境を設定
    pub fn set_active(&mut self, name: Option<String>) {
        if let Some(ref n) = name {
            // 以前のアクティブ環境を非アクティブに
            if let Some(ref old) = self.active {
                if let Some(env) = self.environments.get_mut(old) {
                    env.set_status(EnvironmentStatus::Inactive);
                }
            }
            // 新しい環境をアクティブに
            if let Some(env) = self.environments.get_mut(n) {
                env.set_status(EnvironmentStatus::Active);
            }
        }
        self.active = name;
        self.last_updated = Some(Utc::now());
    }

    /// 環境を取得
    pub fn get(&self, name: &str) -> Option<&AgentEnvironment> {
        self.environments.get(name)
    }

    /// 環境を取得（ミュータブル）
    pub fn get_mut(&mut self, name: &str) -> Option<&mut AgentEnvironment> {
        self.environments.get_mut(name)
    }

    /// アクティブな環境を取得
    pub fn get_active(&self) -> Option<&AgentEnvironment> {
        self.active.as_ref().and_then(|name| self.get(name))
    }
}

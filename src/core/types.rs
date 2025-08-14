#![allow(clippy::all)]
#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    /// 環境がアクティブかどうか
    pub fn is_active(&self) -> bool {
        self.status == EnvironmentStatus::Active
    }

    #[allow(dead_code)]
    /// 環境のパスを取得
    pub fn path(&self) -> &PathBuf {
        &self.worktree_path
    }

    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

impl Config {
    /// 新しい空の設定を作成
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            settings: ConfigSettings::default(),
            path: None,
            global_path: None,
        }
    }
    
    /// デフォルト設定（例を含む）を作成
    pub fn default_example() -> Self {
        // デフォルト設定に例を含める
        let mut settings = ConfigSettings::default();
        
        // 例となるファイルマッピングを追加
        settings.files = vec![
            FileMapping {
                path: PathBuf::from(".env.template"),
                mapping_type: MappingType::Copy,
                description: Some("環境変数設定ファイル".to_string()),
                skip_if_exists: true,
            },
            FileMapping {
                path: PathBuf::from(".claude/config.json"),
                mapping_type: MappingType::Symlink,
                description: Some("Claude設定ファイル".to_string()),
                skip_if_exists: false,
            },
        ];
        
        // 例となるフックを追加
        settings.hooks = HookConfig {
            pre_create: vec![
                HookCommand {
                    command: "echo".to_string(),
                    args: vec!["Creating worktree: {branch}".to_string()],
                    env: HashMap::new(),
                    timeout: 60,
                    continue_on_error: false,
                },
            ],
            post_create: vec![
                HookCommand {
                    command: "echo".to_string(),
                    args: vec!["Worktree created at: {worktree_path}".to_string()],
                    env: HashMap::new(),
                    timeout: 60,
                    continue_on_error: false,
                },
            ],
            pre_remove: vec![],
            post_remove: vec![],
        };
        
        // デフォルトのworktree_baseを設定
        settings.worktree_base = Some(PathBuf::from("../workspaces"));
        
        Self {
            settings,
            path: None,
            global_path: None,
        }
    }

    /// ファイルパスから設定を読み込み
    pub fn from_path(path: &Path) -> crate::core::TwinResult<Self> {
        use std::fs;
        let content = fs::read_to_string(path)?;
        let settings: ConfigSettings =
            toml::from_str(&content).map_err(|e| crate::core::error::TwinError::Config {
                message: format!("Failed to parse config: {}", e),
                path: Some(path.to_path_buf()),
                source: None,
            })?;

        Ok(Self {
            settings,
            path: Some(path.to_path_buf()),
            global_path: None,
        })
    }
}

/// 設定の実際の内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSettings {
    /// Git管理外ファイルの定義
    #[serde(default)]
    pub files: Vec<FileMapping>,

    /// フック設定
    #[serde(default)]
    pub hooks: HookConfig,

    /// Worktreeのベースディレクトリ
    #[serde(default)]
    pub worktree_base: Option<PathBuf>,

    /// デフォルトのブランチプレフィックス
    #[serde(default = "default_branch_prefix")]
    pub branch_prefix: Option<String>,
}

fn default_branch_prefix() -> Option<String> {
    Some("agent".to_string())
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            hooks: HookConfig::default(),
            worktree_base: None,
            branch_prefix: Some("agent".to_string()),
        }
    }
}

/// Git管理外ファイルのマッピング定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    /// ファイルパス（メインリポジトリとワークツリーの両方で同じパス）
    pub path: PathBuf,

    /// マッピングタイプ（symlink or copy）
    #[serde(default = "default_mapping_type")]
    pub mapping_type: MappingType,

    /// 説明（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[allow(dead_code)]

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
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Config関連のテスト
    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.path.is_none());
        assert!(config.global_path.is_none());
        assert_eq!(config.settings.files.len(), 0);
        assert_eq!(config.settings.hooks.pre_create.len(), 0);
        assert_eq!(config.settings.hooks.post_create.len(), 0);
        assert_eq!(config.settings.hooks.pre_remove.len(), 0);
        assert_eq!(config.settings.hooks.post_remove.len(), 0);
        assert_eq!(config.settings.branch_prefix, Some("agent".to_string()));
    }

    #[test]
    fn test_config_from_valid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = r#"
            branch_prefix = "feature"
            
            [[files]]
            path = ".env"
            mapping_type = "copy"
            description = "Environment variables"
            skip_if_exists = true
            
            [[hooks.pre_create]]
            command = "echo 'Creating environment'"
            continue_on_error = false
            timeout = 30
        "#;

        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_path(temp_file.path()).unwrap();
        assert_eq!(config.settings.branch_prefix, Some("feature".to_string()));
        assert_eq!(config.settings.files.len(), 1);
        assert_eq!(config.settings.files[0].path, PathBuf::from(".env"));
        assert_eq!(config.settings.files[0].mapping_type, MappingType::Copy);
        assert_eq!(
            config.settings.files[0].description,
            Some("Environment variables".to_string())
        );
        assert!(config.settings.files[0].skip_if_exists);
        assert_eq!(config.settings.hooks.pre_create.len(), 1);
        assert_eq!(
            config.settings.hooks.pre_create[0].command,
            "echo 'Creating environment'"
        );
        assert_eq!(config.settings.hooks.pre_create[0].timeout, 30);
    }

    #[test]
    fn test_config_from_invalid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid toml { ]").unwrap();

        let result = Config::from_path(temp_file.path());
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                crate::core::error::TwinError::Config { message, .. } => {
                    assert!(message.contains("Failed to parse config"));
                }
                _ => panic!("Expected Config error"),
            }
        }
    }

    #[test]
    fn test_config_settings_default() {
        let settings = ConfigSettings::default();
        assert_eq!(settings.files.len(), 0);
        assert_eq!(settings.branch_prefix, Some("agent".to_string()));
        assert!(settings.worktree_base.is_none());

        // HookConfigもデフォルトで空
        assert_eq!(settings.hooks.pre_create.len(), 0);
        assert_eq!(settings.hooks.post_create.len(), 0);
        assert_eq!(settings.hooks.pre_remove.len(), 0);
        assert_eq!(settings.hooks.post_remove.len(), 0);
    }

    // AgentEnvironment関連のテスト
    #[test]
    fn test_agent_environment_creation() {
        let env = AgentEnvironment::new(
            "test-agent".to_string(),
            "feature/test".to_string(),
            PathBuf::from("/tmp/test-agent"),
            None,
        );

        assert_eq!(env.name, "test-agent");
        assert_eq!(env.branch, "feature/test");
        assert_eq!(env.worktree_path, PathBuf::from("/tmp/test-agent"));
        assert_eq!(env.symlinks.len(), 0);
        assert!(matches!(env.status, EnvironmentStatus::Creating));
        assert!(env.config_path.is_none());
    }

    #[test]
    fn test_agent_environment_is_active() {
        let mut env = AgentEnvironment::new(
            "test".to_string(),
            "test".to_string(),
            PathBuf::from("/tmp"),
            None,
        );

        // Creating状態ではfalse
        assert!(!env.is_active());

        // Active状態ではtrue
        env.status = EnvironmentStatus::Active;
        assert!(env.is_active());

        // Inactive状態ではfalse
        env.status = EnvironmentStatus::Inactive;
        assert!(!env.is_active());

        // Error状態ではfalse
        env.status = EnvironmentStatus::Error("error".to_string());
        assert!(!env.is_active());
    }

    // EnvironmentRegistry関連のテスト
    #[test]
    fn test_registry_add_and_get() {
        let mut registry = EnvironmentRegistry::new();

        let env = AgentEnvironment::new(
            "test1".to_string(),
            "branch1".to_string(),
            PathBuf::from("/tmp/test1"),
            None,
        );

        registry.add(env.clone());

        // 追加したものが取得できる
        let retrieved = registry.get("test1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test1");

        // 存在しないものはNone
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = EnvironmentRegistry::new();

        let env1 = AgentEnvironment::new(
            "test1".to_string(),
            "branch1".to_string(),
            PathBuf::from("/tmp/test1"),
            None,
        );

        let env2 = AgentEnvironment::new(
            "test2".to_string(),
            "branch2".to_string(),
            PathBuf::from("/tmp/test2"),
            None,
        );

        registry.add(env1);
        registry.add(env2);

        assert_eq!(registry.environments.len(), 2);

        // test1を削除
        let removed = registry.remove("test1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "test1");

        // 削除後は1つだけ
        assert_eq!(registry.environments.len(), 1);
        assert!(registry.get("test1").is_none());
        assert!(registry.get("test2").is_some());

        // 存在しないものを削除しようとするとNone
        assert!(registry.remove("nonexistent").is_none());
    }

    // 小さい型のテスト
    #[test]
    fn test_environment_status_display() {
        // EnvironmentStatusは独自のDisplay実装がないため、Debug表示をテスト
        assert_eq!(format!("{:?}", EnvironmentStatus::Active), "Active");
        assert_eq!(format!("{:?}", EnvironmentStatus::Inactive), "Inactive");
        assert_eq!(format!("{:?}", EnvironmentStatus::Creating), "Creating");
        assert_eq!(format!("{:?}", EnvironmentStatus::Removing), "Removing");
        assert_eq!(
            format!("{:?}", EnvironmentStatus::Error("test".to_string())),
            "Error(\"test\")"
        );
    }

    #[test]
    fn test_mapping_type_default() {
        assert_eq!(default_mapping_type(), MappingType::Symlink);

        // Deserializeのデフォルトも確認
        let mapping: FileMapping = serde_json::from_str(r#"{"path": "test.txt"}"#).unwrap();
        assert_eq!(mapping.mapping_type, MappingType::Symlink);
    }

    #[test]
    fn test_file_mapping_skip_if_exists_default() {
        let mapping = FileMapping {
            path: PathBuf::from("test.txt"),
            mapping_type: MappingType::Symlink,
            description: None,
            skip_if_exists: false, // デフォルトはfalse
        };

        assert!(!mapping.skip_if_exists);
    }

    #[test]
    fn test_file_mapping_with_description() {
        let mapping_with = FileMapping {
            path: PathBuf::from("test.txt"),
            mapping_type: MappingType::Copy,
            description: Some("Test file".to_string()),
            skip_if_exists: true,
        };

        let mapping_without = FileMapping {
            path: PathBuf::from("test2.txt"),
            mapping_type: MappingType::Symlink,
            description: None,
            skip_if_exists: false,
        };

        assert_eq!(mapping_with.description, Some("Test file".to_string()));
        assert!(mapping_without.description.is_none());
    }

    #[test]
    fn test_hook_config_default_empty() {
        let config = HookConfig::default();
        assert!(config.pre_create.is_empty());
        assert!(config.post_create.is_empty());
        assert!(config.pre_remove.is_empty());
        assert!(config.post_remove.is_empty());
    }

    #[test]
    fn test_hook_command_creation() {
        let cmd = HookCommand {
            command: "echo test".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout: 60,
            continue_on_error: false,
        };

        assert_eq!(cmd.command, "echo test");
        assert!(!cmd.continue_on_error);
        assert_eq!(cmd.timeout, 60);
        assert!(cmd.env.is_empty());
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn test_symlink_info_states() {
        let mut info = SymlinkInfo::new(PathBuf::from("/source"), PathBuf::from("/target"));

        assert!(!info.is_valid);
        assert!(info.error_message.is_none());

        info.set_success();
        assert!(info.is_valid);
        assert!(info.error_message.is_none());

        info.set_error("Test error".to_string());
        assert!(!info.is_valid);
        assert_eq!(info.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_partial_config_deserialization() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        // 最小限の設定（ファイルのみ）
        let toml_str = r#"
            [[files]]
            path = "test.txt"
        "#;

        writeln!(temp_file, "{}", toml_str).unwrap();

        let config = Config::from_path(temp_file.path()).expect("Should parse minimal config");
        assert_eq!(config.settings.files.len(), 1);
        assert_eq!(config.settings.files[0].path, PathBuf::from("test.txt"));
        assert_eq!(config.settings.files[0].mapping_type, MappingType::Symlink); // デフォルト
        assert!(config.settings.hooks.pre_create.is_empty());
        assert!(config.settings.hooks.post_create.is_empty());
        assert_eq!(config.settings.branch_prefix, Some("agent".to_string())); // デフォルト
    }

    #[test]
    fn test_hooks_only_config() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        // フックのみの設定
        let toml_str = r#"
            [hooks]
            pre_create = [{command = "echo starting"}]
            post_create = [{command = "echo done"}]
        "#;

        writeln!(temp_file, "{}", toml_str).unwrap();

        let config = Config::from_path(temp_file.path()).expect("Should parse hooks-only config");
        assert!(config.settings.files.is_empty());
        assert_eq!(config.settings.hooks.pre_create.len(), 1);
        assert_eq!(config.settings.hooks.pre_create[0].command, "echo starting");
        assert_eq!(config.settings.hooks.post_create.len(), 1);
        assert_eq!(config.settings.hooks.post_create[0].command, "echo done");
        assert!(config.settings.hooks.pre_remove.is_empty());
        assert!(config.settings.hooks.post_remove.is_empty());
    }

    #[test]
    fn test_empty_config() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        // 完全に空の設定
        let toml_str = "";

        writeln!(temp_file, "{}", toml_str).unwrap();

        let config = Config::from_path(temp_file.path()).expect("Should parse empty config");
        assert!(config.settings.files.is_empty());
        assert!(config.settings.hooks.pre_create.is_empty());
        assert!(config.settings.hooks.post_create.is_empty());
        assert!(config.settings.hooks.pre_remove.is_empty());
        assert!(config.settings.hooks.post_remove.is_empty());
        assert_eq!(config.settings.branch_prefix, Some("agent".to_string()));
    }

    #[test]
    fn test_config_with_custom_branch_prefix() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        let toml_str = r#"
            branch_prefix = "feature/"
        "#;

        writeln!(temp_file, "{}", toml_str).unwrap();

        let config = Config::from_path(temp_file.path())
            .expect("Should parse config with custom branch prefix");
        assert_eq!(config.settings.branch_prefix, Some("feature/".to_string()));
    }

    #[test]
    fn test_file_mapping_defaults() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        let toml_str = r#"
            [[files]]
            path = "config.yml"
            # mapping_typeは省略 - デフォルトはsymlink
            # skip_if_existsは省略 - デフォルトはfalse
        "#;

        writeln!(temp_file, "{}", toml_str).unwrap();

        let config = Config::from_path(temp_file.path()).expect("Should parse with defaults");
        assert_eq!(config.settings.files.len(), 1);
        assert_eq!(config.settings.files[0].mapping_type, MappingType::Symlink);
        assert!(!config.settings.files[0].skip_if_exists);
        assert!(config.settings.files[0].description.is_none());
    }

    #[test]
    fn test_hook_command_defaults() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        let toml_str = r#"
            [hooks]
            pre_create = [
                {command = "simple command"}
            ]
        "#;

        writeln!(temp_file, "{}", toml_str).unwrap();

        let config = Config::from_path(temp_file.path()).expect("Should parse hook with defaults");
        let hook = &config.settings.hooks.pre_create[0];
        assert_eq!(hook.command, "simple command");
        assert!(hook.args.is_empty()); // デフォルトは空の配列
        assert!(hook.env.is_empty()); // デフォルトは空のHashMap
        assert_eq!(hook.timeout, 60); // デフォルトタイムアウト
        assert!(!hook.continue_on_error); // デフォルトはfalse
    }
}

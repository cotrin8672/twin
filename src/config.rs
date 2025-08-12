use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// アプリケーション全体の設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// エージェント環境の基本設定
    #[serde(default)]
    pub environment: EnvironmentConfig,
    
    /// シンボリックリンクの設定（source -> target のマッピング）
    #[serde(default)]
    pub symlinks: Vec<SymlinkConfig>,
    
    /// フック設定（環境作成・削除時に実行するコマンド）
    #[serde(default)]
    pub hooks: HookConfig,
    
    /// 自動コミット機能の設定
    #[serde(default)]
    pub auto_commit: AutoCommitConfig,
}

/// 環境設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// worktreeを作成するベースディレクトリ
    pub worktree_base_path: Option<PathBuf>,
    
    /// デフォルトのブランチプレフィックス
    pub branch_prefix: String,
    
    /// 環境レジストリファイルのパス
    pub registry_path: Option<PathBuf>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            worktree_base_path: None,
            branch_prefix: "agent/".to_string(),
            registry_path: None,
        }
    }
}

/// シンボリックリンクの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkConfig {
    /// リンク元のパス（相対パス）
    pub source: PathBuf,
    
    /// リンク先のパス（相対パス）
    pub target: PathBuf,
    
    /// このリンクが必須かどうか
    #[serde(default)]
    pub required: bool,
}

/// フック設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookConfig {
    /// 環境作成前に実行するコマンド
    pub pre_create: Option<Vec<String>>,
    
    /// 環境作成後に実行するコマンド
    pub post_create: Option<Vec<String>>,
    
    /// 環境削除前に実行するコマンド
    pub pre_remove: Option<Vec<String>>,
    
    /// 環境削除後に実行するコマンド
    pub post_remove: Option<Vec<String>>,
}

/// 自動コミット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCommitConfig {
    /// 自動コミット機能を有効にするか
    pub enabled: bool,
    
    /// コミット間隔（秒）
    pub interval_secs: u64,
    
    /// 監視対象のファイルパターン
    pub watch_patterns: Vec<String>,
    
    /// 除外するファイルパターン
    pub ignore_patterns: Vec<String>,
}

impl Default for AutoCommitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 300, // 5分
            watch_patterns: vec!["**/*".to_string()],
            ignore_patterns: vec![
                ".git/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
            ],
        }
    }
}

impl Config {
    /// 設定ファイルを読み込む
    pub async fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }
    
    /// 設定ファイルを保存
    pub async fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        // 親ディレクトリが存在しない場合は作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(path, content)
            .await
            .with_context(|| format!("Failed to write config file: {}", path.display()))
    }
    
    /// デフォルト設定ファイルを作成
    pub fn example() -> Self {
        Self {
            environment: EnvironmentConfig::default(),
            symlinks: vec![
                SymlinkConfig {
                    source: PathBuf::from(".env"),
                    target: PathBuf::from(".env.local"),
                    required: false,
                },
                SymlinkConfig {
                    source: PathBuf::from("config"),
                    target: PathBuf::from("config.local"),
                    required: true,
                },
            ],
            hooks: HookConfig {
                pre_create: Some(vec!["echo 'Creating environment...'".to_string()]),
                post_create: Some(vec!["echo 'Environment created!'".to_string()]),
                pre_remove: Some(vec!["echo 'Removing environment...'".to_string()]),
                post_remove: Some(vec!["echo 'Environment removed!'".to_string()]),
            },
            auto_commit: AutoCommitConfig::default(),
        }
    }
    
    /// グローバル設定とプロジェクト設定をマージ
    pub fn merge(global: Self, project: Self) -> Self {
        // プロジェクト設定を優先し、未設定の項目はグローバル設定を使用
        Self {
            environment: EnvironmentConfig {
                worktree_base_path: project.environment.worktree_base_path
                    .or(global.environment.worktree_base_path),
                branch_prefix: if project.environment.branch_prefix != "agent/" {
                    project.environment.branch_prefix
                } else {
                    global.environment.branch_prefix
                },
                registry_path: project.environment.registry_path
                    .or(global.environment.registry_path),
            },
            symlinks: if !project.symlinks.is_empty() {
                project.symlinks
            } else {
                global.symlinks
            },
            hooks: project.hooks,
            auto_commit: project.auto_commit,
        }
    }
}
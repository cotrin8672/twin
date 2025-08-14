/// 設定管理モジュール
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::core::{FileMapping, HookCommand, HookConfig, MappingType};

/// アプリケーション全体の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Git管理外ファイルのマッピング設定
    #[serde(default)]
    pub files: Vec<FileMapping>,

    /// フック設定（環境作成・削除時に実行するコマンド）
    #[serde(default)]
    pub hooks: HookConfig,

    /// Worktreeのベースディレクトリ
    pub worktree_base: Option<PathBuf>,

    /// デフォルトのブランチプレフィックス
    #[serde(default = "default_branch_prefix")]
    pub branch_prefix: String,
}

fn default_branch_prefix() -> String {
    "agent/".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            hooks: HookConfig::default(),
            worktree_base: None,
            branch_prefix: default_branch_prefix(),
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
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

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
        let mut env_vars = HashMap::new();
        env_vars.insert("NODE_ENV".to_string(), "production".to_string());

        Self {
            files: vec![
                FileMapping {
                    path: PathBuf::from(".env"),
                    mapping_type: MappingType::Symlink,
                    description: Some("環境変数ファイル（共有）".to_string()),
                    skip_if_exists: false,
                },
                FileMapping {
                    path: PathBuf::from(".env.local"),
                    mapping_type: MappingType::Copy,
                    description: Some("ローカル環境変数（各環境で独立）".to_string()),
                    skip_if_exists: false,
                },
                FileMapping {
                    path: PathBuf::from(".vscode/settings.local.json"),
                    mapping_type: MappingType::Symlink,
                    description: Some("VS Codeローカル設定".to_string()),
                    skip_if_exists: true,
                },
            ],
            hooks: HookConfig {
                pre_create: vec![],
                post_create: vec![
                    HookCommand {
                        command: "echo".to_string(),
                        args: vec!["Setting up environment...".to_string()],
                        env: HashMap::new(),
                        timeout: 60,
                        continue_on_error: false,
                    },
                    HookCommand {
                        command: "npm".to_string(),
                        args: vec!["install".to_string()],
                        env: env_vars.clone(),
                        timeout: 300,
                        continue_on_error: false,
                    },
                ],
                pre_remove: vec![HookCommand {
                    command: "echo".to_string(),
                    args: vec!["Cleaning up environment...".to_string()],
                    env: HashMap::new(),
                    timeout: 60,
                    continue_on_error: true,
                }],
                post_remove: vec![],
            },
            worktree_base: Some(PathBuf::from("./worktrees")),
            branch_prefix: "agent/".to_string(),
        }
    }

    /// グローバル設定とプロジェクト設定をマージ
    pub fn merge(global: Self, project: Self) -> Self {
        // プロジェクト設定を優先し、未設定の項目はグローバル設定を使用
        Self {
            files: if !project.files.is_empty() {
                project.files
            } else {
                global.files
            },
            hooks: if project.hooks != HookConfig::default() {
                project.hooks
            } else {
                global.hooks
            },
            worktree_base: project.worktree_base.or(global.worktree_base),
            branch_prefix: if project.branch_prefix != default_branch_prefix() {
                project.branch_prefix
            } else {
                global.branch_prefix
            },
        }
    }

    /// 設定ファイルのパスを取得（プロジェクトルートから検索）
    pub async fn find_config_path(start_path: &Path) -> Option<PathBuf> {
        let mut current = start_path.to_path_buf();

        loop {
            let config_path = current.join("twin.toml");
            if config_path.exists() {
                return Some(config_path);
            }

            let dot_config_path = current.join(".twin.toml");
            if dot_config_path.exists() {
                return Some(dot_config_path);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// グローバル設定ファイルのパスを取得
    pub fn global_config_path() -> Result<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("com", "twin", "twin")
            .context("Failed to get project directories")?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// 設定ファイルを初期化（twin initコマンド用）
    pub async fn init(path: Option<PathBuf>, force: bool) -> Result<PathBuf> {
        let config_path = path.unwrap_or_else(|| PathBuf::from("twin.toml"));

        // ファイルが既に存在する場合
        if config_path.exists() && !force {
            anyhow::bail!(
                "Config file already exists: {}. Use --force to overwrite.",
                config_path.display()
            );
        }

        // サンプル設定を作成
        let config = Self::example();
        config.save(&config_path).await?;

        Ok(config_path)
    }

    /// 設定ファイルまたはグローバル設定を読み込む
    pub async fn load_or_default(project_path: Option<&Path>) -> Result<Self> {
        // プロジェクト設定を探す
        if let Some(path) = project_path {
            if let Some(config_path) = Self::find_config_path(path).await {
                return Self::load(&config_path).await;
            }
        }

        // グローバル設定を試す
        if let Ok(global_path) = Self::global_config_path() {
            if global_path.exists() {
                return Self::load(&global_path).await;
            }
        }

        // デフォルト設定を返す
        Ok(Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.branch_prefix, "agent/");
        assert!(config.files.is_empty());
    }

    #[test]
    fn test_example_config() {
        let config = Config::example();
        assert!(!config.files.is_empty());
        assert_eq!(config.files[0].path, PathBuf::from(".env"));
        assert_eq!(config.files[0].mapping_type, MappingType::Symlink);
        assert_eq!(config.files[1].mapping_type, MappingType::Copy);
        assert!(!config.hooks.post_create.is_empty());
    }

    #[test]
    fn test_hook_command_example() {
        let config = Config::example();
        let first_hook = &config.hooks.post_create[0];
        assert_eq!(first_hook.command, "echo");
        assert_eq!(first_hook.timeout, 60);
        assert!(!first_hook.continue_on_error);
    }
}

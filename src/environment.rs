/// 環境管理モジュール
///
/// このモジュールの役割：
/// - エージェント環境の作成・削除・切り替えを管理
/// - 環境レジストリ（作成済み環境のリスト）の永続化
/// - Gitワークツリーとシンボリックリンクの統合管理
use crate::{
    core::{
        error::TwinError,
        types::{AgentEnvironment, Config, EnvironmentRegistry, EnvironmentStatus, HookCommand},
        TwinResult,
    },
    git::GitManager,
    symlink::{create_symlink_manager, SymlinkManager},
};
use chrono::Utc;
use std::path::{Path, PathBuf};

pub struct EnvironmentManager {
    /// 環境レジストリ
    registry: EnvironmentRegistry,
    /// レジストリファイルのパス
    registry_path: PathBuf,
    /// Git管理
    git: GitManager,
    /// シンボリックリンク管理
    symlink: Box<dyn SymlinkManager>,
    /// 設定
    config: Config,
}

impl EnvironmentManager {
    /// 新しいEnvironmentManagerを作成
    pub fn new(config: Config) -> TwinResult<Self> {
        let git = GitManager::new(std::path::Path::new("."))?;
        let symlink = create_symlink_manager();
        
        // レジストリファイルのパスを決定
        let registry_path = git.get_repo_path().join(".git").join("twin-registry.json");
        
        // 既存のレジストリを読み込むか、新規作成
        let registry = if registry_path.exists() {
            Self::load_registry(&registry_path)?
        } else {
            EnvironmentRegistry::new()
        };
        
        Ok(Self {
            registry,
            registry_path,
            git,
            symlink,
            config,
        })
    }
    
    /// レジストリをファイルから読み込む
    fn load_registry(path: &Path) -> TwinResult<EnvironmentRegistry> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TwinError::Config {
                message: format!("Failed to read registry: {}", e),
                path: Some(path.to_path_buf()),
                source: None,
            })?;
        serde_json::from_str(&content)
            .map_err(|e| TwinError::Config {
                message: format!("Failed to parse registry: {}", e),
                path: Some(path.to_path_buf()),
                source: None,
            })
    }
    
    /// レジストリをファイルに保存
    fn save_registry(&self) -> TwinResult<()> {
        let content = serde_json::to_string_pretty(&self.registry)
            .map_err(|e| TwinError::Config {
                message: format!("Failed to serialize registry: {}", e),
                path: Some(self.registry_path.clone()),
                source: None,
            })?;
        std::fs::write(&self.registry_path, content)
            .map_err(|e| TwinError::Config {
                message: format!("Failed to write registry: {}", e),
                path: Some(self.registry_path.clone()),
                source: None,
            })?;
        Ok(())
    }
    
    /// 環境を作成
    pub fn create_environment(&mut self, name: String, branch: Option<String>) -> TwinResult<AgentEnvironment> {
        // 既存の環境名をチェック
        if self.registry.get(&name).is_some() {
            return Err(TwinError::AlreadyExists {
                resource: "environment".to_string(),
                name: name.clone(),
            });
        }
        
        // ブランチ名を生成または使用
        let branch_name = if let Some(b) = branch {
            b
        } else {
            self.git.generate_agent_branch_name(&name, None)
        };
        
        // ユニークなブランチ名を確保
        let unique_branch = self.git.generate_unique_branch_name(&branch_name, 10)?;
        
        // Worktreeのパスを生成
        let worktree_path = self.git.generate_worktree_path(&name);
        
        // pre_createフックを実行
        if let Some(ref pre_create) = self.config.settings.hooks.pre_create {
            self.execute_hook(pre_create, &name)?;
        }
        
        // Worktreeを作成
        self.git.add_worktree(&worktree_path, Some(&unique_branch), true)?;
        
        // シンボリックリンクを作成
        let mut symlinks = Vec::new();
        if let Some(mappings) = &self.config.settings.file_mappings {
            for mapping in mappings {
                let source = worktree_path.join(&mapping.source);
                let target = PathBuf::from(&mapping.target);
                
                // ターゲットの親ディレクトリを作成
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| TwinError::Io {
                            message: format!("Failed to create parent directory: {}", e),
                            path: parent.to_path_buf(),
                            operation: "create directory".to_string(),
                        })?;
                }
                
                // シンボリックリンクを作成
                self.symlink.create_link(&source, &target)?;
                
                symlinks.push(crate::core::types::SymlinkInfo {
                    source: source.clone(),
                    target: target.clone(),
                    is_valid: true,
                    error_message: None,
                });
            }
        }
        
        // 環境情報を作成
        let env = AgentEnvironment {
            name: name.clone(),
            branch: unique_branch,
            worktree_path,
            symlinks,
            status: EnvironmentStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            config_path: self.config.path.clone(),
        };
        
        // レジストリに追加
        self.registry.add(env.clone());
        self.registry.set_active(Some(name.clone()));
        self.save_registry()?;
        
        // post_createフックを実行
        if let Some(ref post_create) = self.config.settings.hooks.post_create {
            self.execute_hook(post_create, &name)?;
        }
        
        Ok(env)
    }
    
    /// 環境を削除
    pub fn remove_environment(&mut self, name: &str, force: bool) -> TwinResult<()> {
        // 環境を取得
        let env = self.registry.get(name)
            .ok_or_else(|| TwinError::NotFound(format!("Environment '{}'", name)))?
            .clone();
        
        // pre_removeフックを実行
        if let Some(ref pre_remove) = self.config.settings.hooks.pre_remove {
            self.execute_hook(pre_remove, name)?;
        }
        
        // シンボリックリンクを削除
        for symlink in &env.symlinks {
            match self.symlink.remove_symlink(&symlink.target) {
                Ok(_) => {},
                Err(e) if force => {
                    eprintln!("Warning: Failed to remove symlink {}: {}", symlink.target.display(), e);
                },
                Err(e) => return Err(e),
            }
        }
        
        // Worktreeを削除
        match self.git.remove_worktree(&env.worktree_path, force) {
            Ok(_) => {},
            Err(e) if force => {
                eprintln!("Warning: Failed to remove worktree: {}", e);
            },
            Err(e) => return Err(e),
        }
        
        // レジストリから削除
        self.registry.remove(name);
        self.save_registry()?;
        
        // post_removeフックを実行
        if let Some(ref post_remove) = self.config.settings.hooks.post_remove {
            self.execute_hook(post_remove, name)?;
        }
        
        Ok(())
    }
    
    /// フックを実行
    fn execute_hook(&self, hook: &HookCommand, env_name: &str) -> TwinResult<()> {
        use std::process::Command;
        
        let command = hook.command.replace("{name}", env_name);
        
        println!("Executing hook: {}", command);
        
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .output()
        };
        
        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !hook.continue_on_error.unwrap_or(true) {
                        return Err(TwinError::Hook {
                            message: format!("Hook failed: {}", stderr),
                            hook_type: "command".to_string(),
                            exit_code: Some(output.status.code().unwrap_or(-1)),
                        });
                    } else {
                        eprintln!("Warning: Hook failed: {}", stderr);
                    }
                }
                Ok(())
            }
            Err(e) => {
                if !hook.continue_on_error.unwrap_or(true) {
                    Err(TwinError::Hook {
                        message: format!("Failed to execute hook: {}", e),
                        hook_type: "command".to_string(),
                        exit_code: None,
                    })
                } else {
                    eprintln!("Warning: Failed to execute hook: {}", e);
                    Ok(())
                }
            }
        }
    }
}

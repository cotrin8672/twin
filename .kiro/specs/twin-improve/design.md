# Design Document

## Overview

twinは「Git Worktreeの純粋なラッパー + 副作用システム」として設計される。git worktreeの標準的な動作を完全に保持しながら、設定可能な副作用（シンボリックリンク、フック実行等）を自動実行する。自前のレジストリやメタデータ管理は行わず、git worktreeの既存の仕組みを最大限活用することで、シンプルで拡張性の高いアーキテクチャを実現する。

## Architecture

### Core Design Principles

1. **Git Worktree First** - git worktreeの標準動作を最優先
2. **Side Effects Only** - 全ての追加機能は副作用として実装
3. **Zero Learning Curve** - git worktreeの知識がそのまま適用可能
4. **Backward Compatible** - 既存のgit worktreeワークフローを破綻させない
5. **Extensible** - 新しい副作用を簡単に追加可能

### System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Twin CLI                             │
├─────────────────────────────────────────────────────────┤
│  Command Parser  │  Git Worktree Wrapper  │  Output     │
├─────────────────────────────────────────────────────────┤
│                Side Effects Engine                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │
│  │   Symlink   │ │    Hooks    │ │   Future    │       │
│  │   Manager   │ │   Manager   │ │  Extensions │       │
│  └─────────────┘ └─────────────┘ └─────────────┘       │
├─────────────────────────────────────────────────────────┤
│              Configuration System                       │
├─────────────────────────────────────────────────────────┤
│                Git Worktree (Standard)                  │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Command → Command Parser → Git Worktree Operation → Side Effects Engine → Output
                    ↓                      ↓                       ↓
              Config Loader ←─────── Success Check ←──── Effect Execution
```

## Components and Interfaces

### Command Wrapper

```rust
pub struct GitWorktreeWrapper {
    git_cmd: GitCommand,
    side_effects: SideEffectsEngine,
    config: Config,
}

impl GitWorktreeWrapper {
    pub fn add(&self, path: &Path, branch: Option<&str>, options: &[String]) -> Result<()> {
        // 1. Execute git worktree add with exact same arguments
        let result = self.git_cmd.worktree_add(path, branch, options)?;
        
        // 2. Execute post-add side effects
        self.side_effects.execute_post_add(path, &result)?;
        
        Ok(())
    }
    
    pub fn remove(&self, worktree: &Path, options: &[String]) -> Result<()> {
        // 1. Execute pre-remove side effects
        self.side_effects.execute_pre_remove(worktree)?;
        
        // 2. Execute git worktree remove
        self.git_cmd.worktree_remove(worktree, options)?;
        
        // 3. Execute post-remove side effects
        self.side_effects.execute_post_remove(worktree)?;
        
        Ok(())
    }
    
    pub fn list(&self, options: &[String]) -> Result<String> {
        // 1. Get standard git worktree list output
        let mut output = self.git_cmd.worktree_list(options)?;
        
        // 2. Optionally enhance with side effect status
        if self.config.show_side_effect_status {
            output = self.side_effects.enhance_list_output(output)?;
        }
        
        Ok(output)
    }
}
```

### Side Effects Engine

```rust
pub trait SideEffect {
    fn name(&self) -> &str;
    fn execute_pre_add(&self, path: &Path, branch: Option<&str>) -> Result<()>;
    fn execute_post_add(&self, path: &Path, worktree_info: &WorktreeInfo) -> Result<()>;
    fn execute_pre_remove(&self, path: &Path) -> Result<()>;
    fn execute_post_remove(&self, path: &Path) -> Result<()>;
    fn check_status(&self, path: &Path) -> Result<SideEffectStatus>;
    fn repair(&self, path: &Path) -> Result<()>;
}

pub struct SideEffectsEngine {
    effects: Vec<Box<dyn SideEffect>>,
    config: SideEffectsConfig,
}

impl SideEffectsEngine {
    pub fn execute_post_add(&self, path: &Path, worktree_info: &WorktreeInfo) -> Result<()> {
        for effect in &self.effects {
            if self.config.is_enabled(effect.name()) {
                if let Err(e) = effect.execute_post_add(path, worktree_info) {
                    match self.config.error_handling {
                        ErrorHandling::Abort => return Err(e),
                        ErrorHandling::Continue => {
                            eprintln!("Warning: Side effect '{}' failed: {}", effect.name(), e);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
```

### Symlink Side Effect

```rust
pub struct SymlinkSideEffect {
    config: SymlinkConfig,
}

impl SideEffect for SymlinkSideEffect {
    fn name(&self) -> &str { "symlink" }
    
    fn execute_post_add(&self, path: &Path, _worktree_info: &WorktreeInfo) -> Result<()> {
        for link_config in &self.config.links {
            let source = self.resolve_source_path(&link_config.source, path)?;
            let target = path.join(&link_config.target);
            
            self.create_symlink(&source, &target)?;
        }
        Ok(())
    }
    
    fn execute_pre_remove(&self, path: &Path) -> Result<()> {
        // Remove symlinks created by this side effect
        for link_config in &self.config.links {
            let target = path.join(&link_config.target);
            if target.is_symlink() {
                std::fs::remove_file(&target)?;
            }
        }
        Ok(())
    }
    
    fn check_status(&self, path: &Path) -> Result<SideEffectStatus> {
        let mut status = SideEffectStatus::new(self.name());
        
        for link_config in &self.config.links {
            let target = path.join(&link_config.target);
            let source = self.resolve_source_path(&link_config.source, path)?;
            
            if target.exists() && target.is_symlink() {
                if target.read_link()? == source {
                    status.add_ok(format!("Symlink {} -> {} is valid", target.display(), source.display()));
                } else {
                    status.add_error(format!("Symlink {} points to wrong target", target.display()));
                }
            } else {
                status.add_missing(format!("Symlink {} is missing", target.display()));
            }
        }
        
        Ok(status)
    }
}
```

### Hook Side Effect

```rust
pub struct HookSideEffect {
    config: HookConfig,
}

impl SideEffect for HookSideEffect {
    fn name(&self) -> &str { "hooks" }
    
    fn execute_pre_add(&self, path: &Path, branch: Option<&str>) -> Result<()> {
        for hook in &self.config.pre_add {
            self.execute_hook(hook, path, branch)?;
        }
        Ok(())
    }
    
    fn execute_post_add(&self, path: &Path, worktree_info: &WorktreeInfo) -> Result<()> {
        for hook in &self.config.post_add {
            self.execute_hook_with_context(hook, path, worktree_info)?;
        }
        Ok(())
    }
}
```

## Data Models

### Configuration Model

```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub side_effects: SideEffectsConfig,
    pub git_options: GitOptions,
    pub output: OutputConfig,
}

#[derive(Serialize, Deserialize)]
pub struct SideEffectsConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    pub error_handling: ErrorHandling,
    pub symlink: SymlinkConfig,
    pub hooks: HookConfig,
}

#[derive(Serialize, Deserialize)]
pub struct SymlinkConfig {
    pub links: Vec<LinkDefinition>,
}

#[derive(Serialize, Deserialize)]
pub struct LinkDefinition {
    pub source: String,  // Can be relative or absolute
    pub target: String,  // Relative to worktree root
    pub condition: Option<String>, // Optional condition (branch name, path pattern, etc.)
}

#[derive(Serialize, Deserialize)]
pub struct HookConfig {
    pub pre_add: Vec<String>,
    pub post_add: Vec<String>,
    pub pre_remove: Vec<String>,
    pub post_remove: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum ErrorHandling {
    Abort,
    Continue,
}
```

### Status Model

```rust
pub struct SideEffectStatus {
    pub effect_name: String,
    pub items: Vec<StatusItem>,
}

pub struct StatusItem {
    pub status: ItemStatus,
    pub message: String,
}

pub enum ItemStatus {
    Ok,
    Missing,
    Error,
    Warning,
}
```

## Error Handling

### Error Strategy

1. **Git Operation Errors** - 完全にgit worktreeの標準エラーハンドリングに従う
2. **Side Effect Errors** - 設定に基づいて継続またはアボート
3. **Configuration Errors** - 起動時にバリデーションし、明確なエラーメッセージを表示
4. **Partial Failures** - 副作用の部分的失敗を記録し、修復方法を提示

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum TwinError {
    #[error("Git worktree operation failed: {0}")]
    GitWorktreeError(String),
    
    #[error("Side effect '{effect}' failed: {message}")]
    SideEffectError { effect: String, message: String },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Worktree not found: {0}")]
    WorktreeNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Configuration System

### Configuration File Locations

1. **Global Config**: `~/.config/twin/config.toml`
2. **Project Config**: `<project_root>/.twin.toml`
3. **Environment Variables**: `TWIN_*`

### Default Configuration

```toml
[side_effects]
enabled = ["symlink", "hooks"]
error_handling = "continue"

[side_effects.symlink]
[[side_effects.symlink.links]]
source = "../.env"
target = ".env"

[[side_effects.symlink.links]]
source = "../docs"
target = "docs"

[side_effects.hooks]
pre_add = []
post_add = ["echo 'Worktree created: $TWIN_WORKTREE_PATH'"]
pre_remove = []
post_remove = ["echo 'Worktree removed: $TWIN_WORKTREE_PATH'"]

[output]
show_side_effect_status = false
verbose = false
```

## CLI Interface Design

### Command Structure

```bash
# Exact same interface as git worktree
twin add <path> [<commit-ish>] [options...]
twin list [options...]
twin remove <worktree> [options...]
twin prune [options...]

# Additional twin-specific commands
twin status [<worktree>]           # Show side effect status
twin check                         # Check all worktrees for issues
twin repair <worktree>             # Repair side effects for worktree
twin sync                          # Sync side effects for existing worktrees
twin config [key] [value]          # Manage configuration
```

### Output Enhancement

```bash
# Standard git worktree list
$ twin list
/path/to/main      abc1234 [main]
/path/to/feature   def5678 [feature/auth]

# Enhanced output (with --status flag)
$ twin list --status
/path/to/main      abc1234 [main]      ✓ symlinks ✓ hooks
/path/to/feature   def5678 [feature/auth] ✓ symlinks ✗ hooks
```

## Integration with Git Worktree

### Worktree Detection

```rust
pub struct WorktreeDetector {
    repo_root: PathBuf,
}

impl WorktreeDetector {
    pub fn find_all_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        // Use git worktree list --porcelain to get structured output
        let output = Command::new("git")
            .args(&["worktree", "list", "--porcelain"])
            .current_dir(&self.repo_root)
            .output()?;
            
        self.parse_worktree_list(&output.stdout)
    }
    
    pub fn is_worktree_managed_by_twin(&self, path: &Path) -> bool {
        // Check if worktree has twin-managed side effects
        self.has_twin_symlinks(path) || self.has_twin_metadata(path)
    }
}
```

### Backward Compatibility

```rust
pub struct BackwardCompatibility;

impl BackwardCompatibility {
    pub fn migrate_from_old_twin(&self, repo_root: &Path) -> Result<()> {
        // Detect old twin registry files
        // Convert to new side-effect based system
        // Preserve existing worktrees and configurations
        Ok(())
    }
    
    pub fn detect_git_worktrees(&self, repo_root: &Path) -> Result<Vec<PathBuf>> {
        // Find worktrees created by standard git worktree
        // Offer to apply side effects to them
        Ok(vec![])
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_worktree_wrapper_preserves_behavior() {
        let temp_repo = create_test_git_repo();
        let wrapper = GitWorktreeWrapper::new(&temp_repo);
        
        // Test that twin add behaves exactly like git worktree add
        let result = wrapper.add(Path::new("test-worktree"), None, &[]);
        assert!(result.is_ok());
        
        // Verify git worktree list shows the same result
        let git_output = Command::new("git")
            .args(&["worktree", "list"])
            .current_dir(&temp_repo)
            .output()
            .unwrap();
            
        let twin_output = wrapper.list(&[]).unwrap();
        assert_eq!(git_output.stdout, twin_output.as_bytes());
    }

    #[test]
    fn test_side_effects_execution() {
        let temp_repo = create_test_git_repo();
        let config = create_test_config_with_symlinks();
        let wrapper = GitWorktreeWrapper::with_config(&temp_repo, config);
        
        wrapper.add(Path::new("test-worktree"), None, &[]).unwrap();
        
        // Verify symlinks were created
        assert!(Path::new("test-worktree/.env").is_symlink());
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_workflow_compatibility() {
    let temp_repo = create_test_git_repo();
    
    // Create worktree with standard git
    Command::new("git")
        .args(&["worktree", "add", "git-created"])
        .current_dir(&temp_repo)
        .status()
        .unwrap();
    
    // Create worktree with twin
    let wrapper = GitWorktreeWrapper::new(&temp_repo);
    wrapper.add(Path::new("twin-created"), None, &[]).unwrap();
    
    // Both should appear in git worktree list
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .current_dir(&temp_repo)
        .output()
        .unwrap();
    
    let list_str = String::from_utf8(output.stdout).unwrap();
    assert!(list_str.contains("git-created"));
    assert!(list_str.contains("twin-created"));
}
```

## Future Extensibility

### Plugin System Design

```rust
pub trait SideEffectPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn dependencies(&self) -> Vec<&str>;
    fn create_side_effect(&self, config: &Value) -> Box<dyn SideEffect>;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn SideEffectPlugin>>,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn SideEffectPlugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }
    
    pub fn create_side_effects(&self, config: &Config) -> Vec<Box<dyn SideEffect>> {
        // Create side effects based on configuration and available plugins
        vec![]
    }
}
```

### Example Future Extensions

1. **Docker Integration** - コンテナ環境の自動セットアップ
2. **IDE Integration** - エディタ設定の同期
3. **Database Migration** - ブランチ固有のDBスキーマ管理
4. **Environment Variables** - ブランチ固有の環境変数設定
5. **Package Manager** - 依存関係の自動インストール

これらは全て副作用として実装でき、既存のgit worktree操作には一切影響しない。
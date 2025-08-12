# Design Document

## Overview

エージェント開発環境管理ツール「twin」は、Git WorktreeとシンボリックリンクによるクロスプラットフォームなCLIツールです。Rust製で高速動作し、透明性の高い操作でエージェント用の独立環境を管理します。名前の「twin」は、元の環境から分岐した「双子」のような独立環境を作成することを表現しています。

**Note**: RustのTUI実装には成熟した`ratatui`ライブラリを使用。`gitui`、`bottom`等の実績あるツールと同様の技術スタックで実装可能。

## Architecture

### Core Components

```
twin
├── cli/           # CLI interface and argument parsing
├── core/          # Core business logic
│   ├── worktree/  # Git worktree operations
│   ├── symlink/   # Cross-platform symlink management
│   └── config/    # Configuration management
├── platform/      # Platform-specific implementations
└── utils/         # Shared utilities
```

### Data Flow

1. **Environment Creation Flow**
   ```
   User Command → CLI Parser → Config Loader → Worktree Creator → Symlink Manager → Output
   ```

2. **Environment Management Flow**
   ```
   User Command → CLI Parser → Environment Detector → Operation Executor → Output
   ```

## Components and Interfaces

### CLI Interface

```rust
// Main CLI structure
pub enum Command {
    Create {
        name: Option<String>,
        branch: Option<String>,
        cd: bool,
    },
    List {
        format: OutputFormat,
    },
    Remove {
        name: String,
        force: bool,
    },
    Switch {
        name: String,
    },
    Tui,
}

pub enum OutputFormat {
    Table,
    Json,
    Simple,
}
```

### Core Business Logic

```rust
// Environment management
pub struct AgentEnvironment {
    pub name: String,
    pub worktree_path: PathBuf,
    pub branch_name: String,
    pub created_at: DateTime<Utc>,
    pub symlinks: Vec<SymlinkInfo>,
}

pub struct EnvironmentManager {
    config: Config,
    worktree_manager: WorktreeManager,
    symlink_manager: SymlinkManager,
}

impl EnvironmentManager {
    pub fn create_environment(&self, name: &str, base_branch: Option<&str>) -> Result<AgentEnvironment>;
    pub fn list_environments(&self) -> Result<Vec<AgentEnvironment>>;
    pub fn remove_environment(&self, name: &str, force: bool) -> Result<()>;
    pub fn switch_to_environment(&self, name: &str) -> Result<PathBuf>;
}
```

### Worktree Management

```rust
pub struct WorktreeManager {
    repo_root: PathBuf,
    worktree_base: PathBuf,
}

impl WorktreeManager {
    pub fn create_worktree(&self, name: &str, branch: &str) -> Result<PathBuf>;
    pub fn remove_worktree(&self, path: &PathBuf) -> Result<()>;
    pub fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;
}

pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
}
```

### Cross-Platform Symlink Management

```rust
pub trait SymlinkManager {
    fn create_symlink(&self, target: &Path, link: &Path) -> Result<()>;
    fn remove_symlink(&self, link: &Path) -> Result<()>;
    fn is_symlink(&self, path: &Path) -> bool;
}

pub struct UnixSymlinkManager;
pub struct WindowsSymlinkManager;

impl SymlinkManager for UnixSymlinkManager {
    fn create_symlink(&self, target: &Path, link: &Path) -> Result<()> {
        // ln -s implementation
    }
}

impl SymlinkManager for WindowsSymlinkManager {
    fn create_symlink(&self, target: &Path, link: &Path) -> Result<()> {
        // mklink implementation
    }
}
```

## Data Models

### Configuration Model

```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub worktree_base: PathBuf,
    pub symlinks: Vec<SymlinkConfig>,
    pub auto_commit: AutoCommitConfig,
    pub naming: NamingConfig,
}

#[derive(Serialize, Deserialize)]
pub struct SymlinkConfig {
    pub source: PathBuf,
    pub target: String, // relative path in worktree
}

#[derive(Serialize, Deserialize)]
pub struct AutoCommitConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub message_template: String,
}

#[derive(Serialize, Deserialize)]
pub struct NamingConfig {
    pub prefix: String,
    pub timestamp_format: String,
}
```

### Environment State Model

```rust
#[derive(Serialize, Deserialize)]
pub struct EnvironmentState {
    pub environments: HashMap<String, AgentEnvironment>,
    pub last_updated: DateTime<Utc>,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentEnvError {
    #[error("Git operation failed: {0}")]
    GitError(String),
    
    #[error("Symlink operation failed: {0}")]
    SymlinkError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Environment '{0}' not found")]
    EnvironmentNotFound(String),
    
    #[error("Environment '{0}' already exists")]
    EnvironmentExists(String),
    
    #[error("Hook execution failed: {0}")]
    HookError(String),
    
    #[error("Lock acquisition failed: {0}")]
    LockError(String),
    
    #[error("Concurrent operation detected")]
    ConcurrencyError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Error Recovery Strategy

1. **Git Errors**: Display exact git command and error, suggest manual resolution
2. **Symlink Errors**: Show platform-specific command for manual creation (handle Windows admin rights)
3. **Config Errors**: Provide example configuration and validation details
4. **Environment Conflicts**: List existing environments and suggest alternatives
5. **Hook Errors**: Respect `on_hook_failure` setting (abort/continue), show failed command
6. **Lock Errors**: Display waiting message, timeout after reasonable period
7. **Concurrency Errors**: Suggest retry or show conflicting operation details

### Security Considerations

**Local CLI Tool Security Model**:
- User creates own configuration files
- Commands execute with user permissions
- No network operations or external data sources
- Hook commands run in user context (same as manual execution)
- No special security restrictions needed for local development tool

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_environment() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EnvironmentManager::new(temp_dir.path());
        
        let env = manager.create_environment("test-agent", None).unwrap();
        assert_eq!(env.name, "test-agent");
        assert!(env.worktree_path.exists());
    }

    #[test]
    fn test_symlink_creation_unix() {
        // Unix-specific symlink tests
    }

    #[test]
    fn test_symlink_creation_windows() {
        // Windows-specific symlink tests
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_workflow() {
    // Test: create → list → switch → remove
    let temp_repo = create_test_git_repo();
    let manager = EnvironmentManager::new(&temp_repo);
    
    // Create environment
    let env = manager.create_environment("integration-test", None).unwrap();
    
    // Verify it appears in list
    let envs = manager.list_environments().unwrap();
    assert!(envs.iter().any(|e| e.name == "integration-test"));
    
    // Switch to environment
    let path = manager.switch_to_environment("integration-test").unwrap();
    assert_eq!(path, env.worktree_path);
    
    // Remove environment
    manager.remove_environment("integration-test", false).unwrap();
    let envs = manager.list_environments().unwrap();
    assert!(!envs.iter().any(|e| e.name == "integration-test"));
}
```

### Cross-Platform Tests

```rust
#[cfg(target_os = "linux")]
mod linux_tests {
    // Linux-specific tests
}

#[cfg(target_os = "windows")]
mod windows_tests {
    // Windows-specific tests
}
```

## TUI Design

### TUI Architecture

```rust
pub struct TuiApp {
    environments: Vec<AgentEnvironment>,
    selected_index: usize,
    mode: TuiMode,
    manager: EnvironmentManager,
}

pub enum TuiMode {
    List,
    Detail,
    Confirm,
}

impl TuiApp {
    pub fn run(&mut self) -> Result<()> {
        // Main TUI event loop using ratatui
    }
}
```

### TUI Layout

```
┌─ Agent Environments ─────────────────────────────────────┐
│ Name          Branch        Created       Status         │
├───────────────────────────────────────────────────────────┤
│ > agent-001   feature/auth  2024-01-15    Active         │
│   agent-002   main          2024-01-14    Inactive       │
│   agent-003   bugfix/ui     2024-01-13    Active         │
├───────────────────────────────────────────────────────────┤
│ [n] New  [d] Delete  [s] Switch  [q] Quit                │
└───────────────────────────────────────────────────────────┘
```

## Configuration System

### Configuration File Locations

1. **Global Config**: `~/.config/twin/config.toml`
2. **Project Config**: `<project_root>/twin.toml` (following Cargo.toml convention)
3. **Environment Variables**: `TWIN_*`

**File Format**: TOML (extensible to support YAML/JSON in future if needed)

### Default Configuration

```toml
[worktree]
base = "../worktrees"

[naming]
prefix = "agent"
timestamp_format = "%Y%m%d-%H%M%S"

[hooks]
pre_create = []
post_create = []
pre_remove = []
post_remove = []
on_hook_failure = "abort"  # or "continue"

[auto_commit]
enabled = false  # Optional feature, can use ClaudeCode Hooks instead
interval_seconds = 300
message_template = "Auto-commit by {agent_name} at {timestamp}"

[[symlinks]]
source = "../.env"
target = ".env"

[[symlinks]]
source = "../docs"
target = "docs"
```

### Concurrency and Lock Management

**Lock File Strategy**:
- **Location**: `.git/twin.lock` for repository-level locking
- **Purpose**: Prevent concurrent twin operations that could cause conflicts
- **Scope**: Environment creation, deletion, and registry updates
- **Implementation**: File-based locking with automatic cleanup on process exit

**Potential Conflicts**:
1. Simultaneous environment name generation
2. Concurrent environment registry file updates
3. Git worktree operations on the same repository
4. Symlink creation/deletion conflicts

**Lock Granularity**: Repository-level locking for simplicity and safety

## CLI Command Examples

```bash
# Create new environment
twin create                    # Auto-generated name
twin create my-agent           # Specific name
twin create --cd my-agent      # Create and cd

# List environments
twin list                      # Table format
twin list --format json       # JSON format

# Switch environment
twin switch my-agent           # Output cd command

# Remove environment
twin remove my-agent           # With confirmation
twin remove my-agent --force   # Skip confirmation

# TUI mode
twin tui                       # Interactive mode
```
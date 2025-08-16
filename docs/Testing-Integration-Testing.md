# Integration Testing

Integration tests verify that Twin's Git worktree operations and their effects work together correctly.

## Test Structure

Integration tests are located in the `tests/` directory and test complete effect chains:

```rust
// tests/integration_test.rs
use twin::{GitManager, EffectChain, Config};
use tempfile::tempdir;

#[test]
fn test_worktree_creation_with_effects() {
    let temp = tempdir().unwrap();
    let repo = common::init_test_repo(temp.path());
    
    // Load configuration with effects
    let config = Config::from_str(r#"
        [[files]]
        source = "template.txt"
        target = "config.txt"
        mapping_type = "symlink"
        
        [hooks]
        post_create = [
            { command = "echo 'Worktree created'" }
        ]
    "#).unwrap();
    
    // Create worktree with effects
    let manager = GitManager::new(temp.path()).unwrap();
    let result = manager.create_worktree_with_effects(
        "feature-branch",
        "../feature-branch",
        &config
    );
    
    assert!(result.is_ok());
    
    // Verify primary operation
    assert!(Path::new("../feature-branch").exists());
    
    // Verify effects were applied
    assert!(Path::new("../feature-branch/config.txt").exists());
}
```

## Testing Effect Chains

```rust
// tests/effect_chain_test.rs
#[test]
fn test_effect_chain_execution() {
    let temp = tempdir().unwrap();
    let context = create_test_context(&temp);
    
    // Create effect chain
    let effects = vec![
        create_symlink_effect("source1.txt", "target1.txt"),
        create_symlink_effect("source2.txt", "target2.txt"),
        create_hook_effect("touch marker.txt"),
    ];
    
    let chain = EffectChain::new(effects);
    let results = chain.execute(&context).unwrap();
    
    // Verify all effects executed
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_success()));
    
    // Verify effect outcomes
    assert!(context.worktree_path.join("target1.txt").exists());
    assert!(context.worktree_path.join("target2.txt").exists());
    assert!(context.worktree_path.join("marker.txt").exists());
}
```

## Testing Effect Failure Recovery

```rust
#[test]
fn test_effect_chain_continues_on_failure() {
    let temp = tempdir().unwrap();
    let context = create_test_context(&temp);
    
    // Create chain with failing effect in middle
    let effects = vec![
        create_symlink_effect("source.txt", "target1.txt"),
        create_failing_effect("This will fail"),
        create_hook_effect("touch success_marker.txt"),
    ];
    
    let chain = EffectChain::new(effects);
    let results = chain.execute(&context).unwrap();
    
    // Verify execution continued after failure
    assert_eq!(results.len(), 3);
    assert!(results[0].is_success());
    assert!(!results[1].is_success());
    assert!(results[2].is_success());
    
    // Verify successful effects still applied
    assert!(context.worktree_path.join("target1.txt").exists());
    assert!(context.worktree_path.join("success_marker.txt").exists());
}
```

## Testing Platform-Specific Effect Behavior

```rust
// tests/platform_effects_test.rs
#[test]
fn test_cross_platform_symlink_effects() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("source.txt");
    std::fs::write(&source, "content").unwrap();
    
    let manager = symlink::create_platform_manager();
    let target = temp.path().join("target.txt");
    
    let result = manager.create_link(&source, &target);
    assert!(result.is_ok());
    
    // Verify effect worked regardless of platform
    assert!(target.exists());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "content");
    
    #[cfg(unix)]
    assert!(std::fs::symlink_metadata(&target).unwrap().file_type().is_symlink());
    
    #[cfg(windows)]
    {
        // On Windows, might be symlink or copy depending on permissions
        let metadata = std::fs::symlink_metadata(&target).unwrap();
        assert!(metadata.file_type().is_symlink() || metadata.file_type().is_file());
    }
}
```

## Testing Hook Effect Integration

```rust
// tests/hook_integration_test.rs
#[test]
fn test_hook_effects_with_worktree_context() {
    let temp = tempdir().unwrap();
    let repo = common::init_test_repo(temp.path());
    
    let config = Config {
        hooks: Some(HookConfig {
            pre_create: vec![
                HookCommand {
                    command: "echo 'Pre: {branch}' > pre.txt".to_string(),
                    continue_on_error: false,
                    timeout: Some(60),
                }
            ],
            post_create: vec![
                HookCommand {
                    command: "echo 'Post: {worktree_path}' > {worktree_path}/post.txt".to_string(),
                    continue_on_error: false,
                    timeout: Some(60),
                }
            ],
            ..Default::default()
        }),
        ..Default::default()
    };
    
    let manager = GitManager::new(temp.path()).unwrap();
    manager.create_worktree_with_effects("test-branch", "../test-branch", &config).unwrap();
    
    // Verify pre-create hook effect
    let pre_content = std::fs::read_to_string(temp.path().join("pre.txt")).unwrap();
    assert!(pre_content.contains("Pre: test-branch"));
    
    // Verify post-create hook effect
    let post_content = std::fs::read_to_string("../test-branch/post.txt").unwrap();
    assert!(post_content.contains("Post:"));
    assert!(post_content.contains("test-branch"));
}
```

## Testing Complete Workflows

```rust
// tests/workflow_test.rs
#[test]
fn test_complete_create_remove_workflow() {
    let temp = tempdir().unwrap();
    let repo = common::init_test_repo(temp.path());
    
    let config = load_test_config();
    let manager = GitManager::new(temp.path()).unwrap();
    
    // Create worktree with all effects
    let create_result = manager.create_worktree_with_effects(
        "feature", 
        "../feature",
        &config
    );
    assert!(create_result.is_ok());
    
    // Verify creation effects
    assert!(Path::new("../feature").exists());
    assert!(Path::new("../feature/.env").exists()); // File effect
    assert!(Path::new("../feature/node_modules").exists()); // Hook effect
    
    // Remove worktree with cleanup effects
    let remove_result = manager.remove_worktree_with_effects(
        "feature",
        &config
    );
    assert!(remove_result.is_ok());
    
    // Verify removal effects
    assert!(!Path::new("../feature").exists());
    assert!(!Path::new("../feature/.env").exists());
}
```

## Testing Effect Ordering

```rust
#[test]
fn test_effect_execution_order() {
    let temp = tempdir().unwrap();
    let order_file = temp.path().join("order.txt");
    
    let effects = vec![
        create_hook_effect(&format!("echo '1' >> {}", order_file.display())),
        create_hook_effect(&format!("echo '2' >> {}", order_file.display())),
        create_hook_effect(&format!("echo '3' >> {}", order_file.display())),
    ];
    
    let chain = EffectChain::new(effects);
    chain.execute(&create_test_context(&temp)).unwrap();
    
    let content = std::fs::read_to_string(&order_file).unwrap();
    assert_eq!(content.trim(), "1\n2\n3");
}
```

## Common Test Module

```rust
// tests/common/mod.rs
pub fn init_test_repo(path: &Path) -> git2::Repository {
    let repo = git2::Repository::init(path).unwrap();
    commit_initial_file(&repo);
    repo
}

pub fn create_test_context(temp: &TempDir) -> WorktreeContext {
    WorktreeContext {
        branch_name: "test-branch".to_string(),
        worktree_path: temp.path().join("worktree"),
        source_path: temp.path().to_path_buf(),
    }
}

pub fn load_test_config() -> Config {
    Config {
        worktree_base: "../test-worktrees".to_string(),
        files: vec![
            FileMapping {
                source: ".env.template".to_string(),
                target: ".env".to_string(),
                mapping_type: MappingType::Copy,
                skip_if_exists: false,
                description: Some("Test environment".to_string()),
            }
        ],
        hooks: Some(HookConfig {
            post_create: vec![
                HookCommand {
                    command: "mkdir -p node_modules".to_string(),
                    continue_on_error: true,
                    timeout: Some(60),
                }
            ],
            ..Default::default()
        }),
    }
}
```

## Running Integration Tests

```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test file
cargo test --test integration_test

# Run with logging for debugging
RUST_LOG=debug cargo test --test '*' -- --nocapture

# Run integration tests for specific effect type
cargo test --test hook_integration_test
```

Source: [tests/integration_test.rs](https://github.com/your-org/twin/blob/main/tests/integration_test.rs), [tests/common/mod.rs](https://github.com/your-org/twin/blob/main/tests/common/mod.rs)
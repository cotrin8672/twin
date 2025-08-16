# End-to-End Testing

E2E tests verify complete user workflows from command-line input to effect execution and output.

## Testing Complete User Scenarios

E2E tests simulate real user interactions with Twin:

```rust
// tests/e2e_basic.rs
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_create_with_effects() {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp).unwrap();
    
    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .output()
        .expect("Failed to init git");
    
    // Create config file with effects
    std::fs::write("twin.toml", r#"
        [[files]]
        source = "README.md"
        target = "README.md"
        mapping_type = "symlink"
        
        [hooks]
        post_create = [
            { command = "echo 'Created' > created.txt" }
        ]
    "#).unwrap();
    
    // Create initial README
    std::fs::write("README.md", "# Test Project").unwrap();
    
    // Run twin create command
    let output = Command::new("twin")
        .args(&["create", "feature-test"])
        .output()
        .expect("Failed to execute twin");
    
    assert!(output.status.success());
    
    // Verify worktree created
    assert!(Path::new("../feature-test").exists());
    
    // Verify symlink effect applied
    assert!(Path::new("../feature-test/README.md").exists());
    
    // Verify hook effect executed
    assert!(Path::new("../feature-test/created.txt").exists());
}
```

## Testing Output Formats with Effects

```rust
#[test]
fn test_json_output_includes_effects() {
    let output = Command::new("twin")
        .args(&["list", "--format", "json"])
        .output()
        .unwrap();
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    
    // Verify JSON includes effect information
    assert!(json.is_array());
    if let Some(first) = json.as_array().unwrap().first() {
        assert!(first.get("branch").is_some());
        assert!(first.get("path").is_some());
        assert!(first.get("effects_applied").is_some());
    }
}

#[test]
fn test_table_output_shows_effect_status() {
    // Create worktree with effects
    Command::new("twin")
        .args(&["create", "test-branch"])
        .output()
        .unwrap();
    
    let output = Command::new("twin")
        .args(&["list"])
        .output()
        .unwrap();
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Table should show effect status
    assert!(output_str.contains("Branch"));
    assert!(output_str.contains("Path"));
    assert!(output_str.contains("Effects"));
}
```

## Testing Effect Error Scenarios

```rust
#[test]
fn test_effect_failure_handling() {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp).unwrap();
    
    // Create config with failing effect
    std::fs::write("twin.toml", r#"
        [[files]]
        source = "nonexistent.txt"
        target = "target.txt"
        mapping_type = "symlink"
        
        [hooks]
        post_create = [
            { command = "exit 1", continue_on_error = false }
        ]
    "#).unwrap();
    
    let output = Command::new("twin")
        .args(&["create", "test"])
        .output()
        .unwrap();
    
    // Command should still succeed (worktree created)
    assert!(output.status.success());
    
    // But stderr should contain effect warnings
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("effect") || stderr.contains("warning"));
}
```

## Testing Cross-Platform Effect Behavior

```rust
#[test]
fn test_platform_agnostic_effects() {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp).unwrap();
    
    // Initialize repo
    Command::new("git").args(&["init"]).output().unwrap();
    
    // Create cross-platform config
    std::fs::write("twin.toml", r#"
        [[files]]
        source = "config.txt"
        target = "config.txt"
        mapping_type = "symlink"  # Falls back to copy on Windows without perms
    "#).unwrap();
    
    std::fs::write("config.txt", "test content").unwrap();
    
    let output = Command::new("twin")
        .args(&["create", "cross-platform"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // Verify effect worked regardless of platform
    let target = Path::new("../cross-platform/config.txt");
    assert!(target.exists());
    
    let content = std::fs::read_to_string(target).unwrap();
    assert_eq!(content, "test content");
}
```

## Container-Based E2E Testing

For complete isolation and cross-platform testing:

```rust
// tests/e2e_container.rs
use testcontainers::{clients, images::generic::GenericImage};

#[test]
fn test_effects_in_container() {
    let docker = clients::Cli::default();
    
    // Create container with Git and Rust
    let container = docker.run(
        GenericImage::new("rust", "latest")
            .with_env_var("RUST_LOG", "debug")
            .with_volume(("/tmp/test", "/workspace"))
    );
    
    // Set up test repository
    container.exec(vec![
        "sh", "-c", 
        "cd /workspace && git init && git config user.email 'test@test.com' && git config user.name 'Test'"
    ]);
    
    // Create config with effects
    container.exec(vec![
        "sh", "-c",
        r#"cd /workspace && cat > twin.toml << EOF
[[files]]
source = "test.txt"
target = "test.txt"
mapping_type = "symlink"

[hooks]
post_create = [{ command = "touch /workspace/effect_marker" }]
EOF"#
    ]);
    
    // Create test file
    container.exec(vec!["sh", "-c", "cd /workspace && echo 'test' > test.txt"]);
    
    // Install and run twin
    container.exec(vec![
        "sh", "-c",
        "cd /workspace && cargo install --path /twin && twin create feature"
    ]);
    
    // Verify effects in container
    let symlink_check = container.exec(vec![
        "sh", "-c", 
        "test -L /workspace/../feature/test.txt && echo 'symlink exists'"
    ]);
    assert!(symlink_check.contains("symlink exists"));
    
    let hook_check = container.exec(vec![
        "sh", "-c",
        "test -f /workspace/effect_marker && echo 'hook executed'"
    ]);
    assert!(hook_check.contains("hook executed"));
}
```

## Testing Effect Performance

```rust
#[test]
fn test_effect_performance_e2e() {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp).unwrap();
    
    // Create config with many effects
    let mut config = String::from("");
    for i in 0..50 {
        config.push_str(&format!(r#"
[[files]]
source = "file{}.txt"
target = "file{}.txt"
mapping_type = "copy"
"#, i, i));
    }
    
    std::fs::write("twin.toml", config).unwrap();
    
    // Create source files
    for i in 0..50 {
        std::fs::write(format!("file{}.txt", i), "content").unwrap();
    }
    
    Command::new("git").args(&["init"]).output().unwrap();
    
    let start = std::time::Instant::now();
    
    let output = Command::new("twin")
        .args(&["create", "perf-test"])
        .output()
        .unwrap();
    
    let duration = start.elapsed();
    
    assert!(output.status.success());
    assert!(duration.as_secs() < 10, "Too slow with many effects: {:?}", duration);
    
    // Verify all effects applied
    for i in 0..50 {
        assert!(Path::new(&format!("../perf-test/file{}.txt", i)).exists());
    }
}
```

## Testing User Workflows

```rust
#[test]
fn test_developer_workflow() {
    // Simulate typical developer workflow
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp).unwrap();
    
    // 1. Initialize project
    Command::new("git").args(&["init"]).output().unwrap();
    
    // 2. Set up Twin configuration
    std::fs::write("twin.toml", r#"
        worktree_base = "../features"
        
        [[files]]
        source = ".env.example"
        target = ".env"
        mapping_type = "copy"
        
        [hooks]
        post_create = [
            { command = "npm install", continue_on_error = true }
        ]
    "#).unwrap();
    
    std::fs::write(".env.example", "API_KEY=test").unwrap();
    std::fs::write("package.json", "{}").unwrap();
    
    // 3. Create feature branch with effects
    let create_output = Command::new("twin")
        .args(&["create", "feature-auth", "--print-path"])
        .output()
        .unwrap();
    
    assert!(create_output.status.success());
    let path = String::from_utf8_lossy(&create_output.stdout);
    assert!(path.contains("features/feature-auth"));
    
    // 4. List worktrees
    let list_output = Command::new("twin")
        .args(&["list"])
        .output()
        .unwrap();
    
    assert!(String::from_utf8_lossy(&list_output.stdout).contains("feature-auth"));
    
    // 5. Remove worktree
    let remove_output = Command::new("twin")
        .args(&["remove", "feature-auth"])
        .output()
        .unwrap();
    
    assert!(remove_output.status.success());
    assert!(!Path::new("../features/feature-auth").exists());
}
```

## Testing CLI Arguments with Effects

```rust
#[test]
fn test_cli_arguments_affect_effects() {
    // Test --no-effects flag (future feature)
    let output = Command::new("twin")
        .args(&["create", "test", "--no-hooks"])
        .env("TWIN_SKIP_HOOKS", "1")
        .output()
        .unwrap();
    
    // Verify hooks were skipped
    assert!(!Path::new("../test/hook_marker.txt").exists());
    
    // Test custom config
    let output = Command::new("twin")
        .args(&["create", "test2", "--config", "custom.toml"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
}
```

Source: [tests/e2e_basic.rs](https://github.com/your-org/twin/blob/main/tests/e2e_basic.rs)
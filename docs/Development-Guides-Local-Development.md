# Local Development

This guide covers day-to-day development workflows for Twin, focusing on implementing and testing effects.

## Development Workflow

### 1. Understanding the Effect System

Before making changes, understand Twin's core concept:
- **Primary**: Git worktree operations
- **Effects**: Everything else (symlinks, hooks, configs)

```bash
# Explore the effect modules
ls src/symlink.rs  # Symlink effects
ls src/hooks.rs     # Hook effects
ls src/config.rs    # Effect configuration
```

### 2. Setting Up Your Development Branch

```bash
# Traditional approach
git checkout -b feature/new-effect

# Or use Twin itself to develop Twin!
twin create feature/new-effect
cd ../feature/new-effect
```

### 3. Running During Development

#### Quick Testing
```bash
# Build and run immediately
cargo run -- create test-branch

# With debug output to see effects
TWIN_DEBUG=1 cargo run -- create test-branch

# Test specific effect configurations
cargo run -- create test --config test-effects.toml
```

#### Testing Effects
```bash
# Test effect in isolation
cargo test symlink_effect

# Test effect chain
cargo test effect_chain

# Integration test with effects
cargo test --test integration_test
```

## Adding a New Effect Type

### 1. Define the Effect

Create a new module for your effect:

```rust
// src/permission_effect.rs
use crate::core::types::WorktreeContext;

pub struct PermissionEffect {
    path: PathBuf,
    mode: u32,
}

impl Effect for PermissionEffect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        let target = context.worktree_path.join(&self.path);
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&target)?.permissions();
            perms.set_mode(self.mode);
            std::fs::set_permissions(&target, perms)?;
        }
        
        Ok(EffectResult::success("Permissions set"))
    }
    
    fn can_apply(&self, context: &WorktreeContext) -> bool {
        context.worktree_path.join(&self.path).exists()
    }
    
    fn effect_type(&self) -> &str {
        "permission"
    }
}
```

### 2. Add Configuration Support

Update the configuration to support your effect:

```rust
// src/core/types.rs
#[derive(Debug, Deserialize)]
pub struct FileMapping {
    pub source: String,
    pub target: String,
    pub mapping_type: MappingType,
    pub permissions: Option<u32>,  // New field for permission effect
}
```

### 3. Wire into Effect Chain

Add your effect to the effect chain:

```rust
// src/cli/commands.rs
fn build_effect_chain(config: &Config, context: &WorktreeContext) -> EffectChain {
    let mut effects: Vec<Box<dyn Effect>> = Vec::new();
    
    // Existing effects
    for file in &config.files {
        effects.push(Box::new(SymlinkEffect::from(file)));
        
        // Add permission effect if specified
        if let Some(mode) = file.permissions {
            effects.push(Box::new(PermissionEffect::new(&file.target, mode)));
        }
    }
    
    EffectChain::new(effects)
}
```

### 4. Test Your Effect

```rust
// src/permission_effect.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_effect() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, "content").unwrap();
        
        let effect = PermissionEffect::new("test.txt", 0o755);
        let context = WorktreeContext {
            worktree_path: temp.path().to_path_buf(),
            ..Default::default()
        };
        
        let result = effect.apply(&context).unwrap();
        assert!(result.is_success());
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::metadata(&file).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o755);
        }
    }
}
```

## Debugging Effects

### Enable Detailed Logging

```rust
// Add debug output to your effect
impl Effect for CustomEffect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        debug!("Applying custom effect to: {:?}", context.worktree_path);
        
        // Effect implementation
        let result = do_something()?;
        
        debug!("Custom effect result: {:?}", result);
        Ok(EffectResult::success("Custom effect applied"))
    }
}
```

### Test with Verbose Output

```bash
# See all effect execution
RUST_LOG=twin=debug cargo run -- create test

# See specific effect module
RUST_LOG=twin::symlink=debug cargo run -- create test

# Full backtrace on errors
RUST_BACKTRACE=1 cargo run -- create test
```

## Working with Platform-Specific Effects

### Conditional Compilation

```rust
// Platform-specific effect implementation
impl PlatformEffect {
    #[cfg(windows)]
    fn apply_windows(&self, context: &WorktreeContext) -> Result<EffectResult> {
        // Windows-specific implementation
    }
    
    #[cfg(unix)]
    fn apply_unix(&self, context: &WorktreeContext) -> Result<EffectResult> {
        // Unix-specific implementation
    }
    
    pub fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        #[cfg(windows)]
        return self.apply_windows(context);
        
        #[cfg(unix)]
        return self.apply_unix(context);
    }
}
```

### Testing Across Platforms

```bash
# Test on current platform
cargo test

# Cross-compile and test (requires setup)
cargo test --target x86_64-pc-windows-gnu
cargo test --target x86_64-unknown-linux-gnu
```

## Effect Development Best Practices

### 1. Effects Should Be Idempotent

```rust
impl Effect for IdempotentEffect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        // Check if already applied
        if self.is_already_applied(context) {
            return Ok(EffectResult::skipped("Already applied"));
        }
        
        // Apply the effect
        self.do_apply(context)
    }
}
```

### 2. Effects Should Handle Failures Gracefully

```rust
impl Effect for ResilientEffect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        match self.try_primary_method(context) {
            Ok(result) => Ok(result),
            Err(e) if e.is_recoverable() => {
                warn!("Primary method failed, trying fallback: {}", e);
                self.try_fallback_method(context)
            }
            Err(e) => Err(e),
        }
    }
}
```

### 3. Effects Should Be Testable

```rust
// Make effects easy to test
pub struct TestableEffect {
    // Inject dependencies for testing
    executor: Box<dyn CommandExecutor>,
}

impl TestableEffect {
    #[cfg(test)]
    pub fn with_mock_executor(executor: Box<dyn CommandExecutor>) -> Self {
        Self { executor }
    }
}
```

## Local Testing Workflow

```bash
# 1. Make changes to effects
vim src/my_effect.rs

# 2. Check compilation
cargo check

# 3. Run effect-specific tests
cargo test my_effect

# 4. Test integration
cargo test --test integration_test

# 5. Manual testing
cat > test.toml << EOF
[[files]]
source = "test.txt"
target = "test.txt"
my_custom_property = true
EOF

cargo run -- create test-branch --config test.toml

# 6. Verify effects
ls -la ../test-branch/
```

## Pre-commit Checks

Before committing effect changes:

```bash
# Format code
cargo fmt

# Check lints
cargo clippy -- -D warnings

# Run all tests
cargo test

# Check documentation
cargo doc --no-deps

# Verify examples
cargo test --doc
```

Source: [src/main.rs](https://github.com/your-org/twin/blob/main/src/main.rs), [src/cli/](https://github.com/your-org/twin/blob/main/src/cli/)
# Unit Testing

Unit tests in Twin focus on testing individual effects and their components in isolation.

## Running Unit Tests

```bash
# Run all unit tests
cargo test --lib

# Run specific effect tests
cargo test --lib symlink
cargo test --lib hook
cargo test --lib config

# Run with output for debugging effects
cargo test --lib -- --nocapture

# Run single effect test
cargo test --lib test_symlink_effect_creation
```

## Testing Effect Components

### Testing Individual Effects

Each effect type should be tested independently:

```rust
// src/symlink.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symlink_effect_creation() {
        let effect = SymlinkEffect::new(
            PathBuf::from("source.txt"),
            PathBuf::from("target.txt")
        );
        
        assert_eq!(effect.source(), Path::new("source.txt"));
        assert_eq!(effect.target(), Path::new("target.txt"));
        assert_eq!(effect.effect_type(), EffectType::Symlink);
    }

    #[test]
    fn test_symlink_effect_validation() {
        let effect = SymlinkEffect::new(
            PathBuf::from("nonexistent.txt"),
            PathBuf::from("target.txt")
        );
        
        let context = WorktreeContext::test_context();
        assert!(!effect.can_apply(&context));
    }
}
```

### Testing Effect Strategies

Platform-specific effect strategies need separate tests:

```rust
// src/symlink.rs
#[cfg(test)]
mod platform_tests {
    use super::*;

    #[test]
    fn test_platform_effect_selection() {
        let strategy = select_symlink_strategy();
        
        #[cfg(unix)]
        assert!(matches!(strategy, SymlinkStrategy::Native));
        
        #[cfg(windows)]
        {
            if has_developer_mode() {
                assert!(matches!(strategy, SymlinkStrategy::Native));
            } else {
                assert!(matches!(strategy, SymlinkStrategy::Copy));
            }
        }
    }
}
```

### Testing Hook Effects

Hook effects require testing of command parsing and execution:

```rust
// src/hooks.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_variable_substitution() {
        let hook = HookCommand {
            command: "echo Branch: {branch}, Path: {worktree_path}".to_string(),
            continue_on_error: false,
            timeout: Some(60),
        };
        
        let context = WorktreeContext {
            branch_name: "feature-123".to_string(),
            worktree_path: PathBuf::from("/path/to/worktree"),
            source_path: PathBuf::from("/path/to/source"),
        };
        
        let prepared = hook.prepare(&context);
        assert_eq!(
            prepared,
            "echo Branch: feature-123, Path: /path/to/worktree"
        );
    }

    #[test]
    fn test_hook_timeout_handling() {
        let hook = HookCommand {
            command: "sleep 10".to_string(),
            continue_on_error: false,
            timeout: Some(1), // 1 second timeout
        };
        
        let result = hook.execute(&WorktreeContext::test_context());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TwinError::Timeout(_)));
    }
}
```

## Testing Effect Configuration

```rust
// src/config.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_config_parsing() {
        let toml = r#"
            [[files]]
            source = "config.template"
            target = ".config"
            mapping_type = "symlink"
            
            [hooks]
            post_create = [
                { command = "npm install", timeout = 300 }
            ]
        "#;
        
        let config: Config = toml::from_str(toml).unwrap();
        
        // Verify file effects
        assert_eq!(config.files.len(), 1);
        assert_eq!(config.files[0].mapping_type, MappingType::Symlink);
        
        // Verify hook effects
        assert!(config.hooks.is_some());
        let hooks = config.hooks.unwrap();
        assert_eq!(hooks.post_create.len(), 1);
    }

    #[test]
    fn test_invalid_effect_config() {
        let toml = r#"
            [[files]]
            source = "config.template"
            # missing target - should fail
            mapping_type = "invalid"
        "#;
        
        let result: Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err());
    }
}
```

## Testing Effect Errors

```rust
// src/core/error.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_error_categorization() {
        let error = TwinError::EffectError {
            effect_type: "symlink".to_string(),
            recoverable: true,
            message: "Permission denied".to_string(),
        };
        
        assert!(error.is_recoverable());
        assert_eq!(error.effect_type(), Some("symlink"));
    }

    #[test]
    fn test_critical_vs_effect_errors() {
        let critical = TwinError::GitOperationError("No repository".to_string());
        let effect = TwinError::EffectWarning("Optional hook failed".to_string());
        
        assert!(critical.is_critical());
        assert!(!effect.is_critical());
    }
}
```

## Mocking Effects for Tests

```rust
use mockall::{automock, predicate::*};

#[automock]
trait Effect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult>;
    fn can_apply(&self, context: &WorktreeContext) -> bool;
    fn rollback(&self, context: &WorktreeContext) -> Result<()>;
}

#[test]
fn test_effect_orchestration_with_mocks() {
    let mut mock_effect = MockEffect::new();
    
    // Set up expectations
    mock_effect.expect_can_apply()
        .returning(|_| true);
    
    mock_effect.expect_apply()
        .times(1)
        .returning(|_| Ok(EffectResult::success("Mock effect applied")));
    
    // Test the orchestration logic
    let chain = EffectChain::new(vec![Box::new(mock_effect)]);
    let results = chain.execute(&WorktreeContext::test_context());
    
    assert_eq!(results.len(), 1);
    assert!(results[0].is_success());
}
```

## Test Helpers for Effects

```rust
// src/test_helpers.rs
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    
    pub struct EffectTestBuilder {
        context: WorktreeContext,
        effects: Vec<Box<dyn Effect>>,
    }
    
    impl EffectTestBuilder {
        pub fn new() -> Self {
            Self {
                context: WorktreeContext::test_context(),
                effects: Vec::new(),
            }
        }
        
        pub fn with_symlink(mut self, source: &str, target: &str) -> Self {
            self.effects.push(Box::new(
                SymlinkEffect::new(source.into(), target.into())
            ));
            self
        }
        
        pub fn with_hook(mut self, command: &str) -> Self {
            self.effects.push(Box::new(
                HookEffect::new(command.to_string())
            ));
            self
        }
        
        pub fn execute(self) -> Vec<EffectResult> {
            EffectChain::new(self.effects).execute(&self.context)
        }
    }
}

#[test]
fn test_effect_builder() {
    let results = EffectTestBuilder::new()
        .with_symlink("source.txt", "target.txt")
        .with_hook("echo test")
        .execute();
    
    assert_eq!(results.len(), 2);
}
```

## Coverage Guidelines for Effects

- Each effect type should have >90% coverage
- All effect error paths must be tested
- Platform-specific effect code needs conditional tests
- Effect interaction points require integration tests

## Running Coverage Reports

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run coverage for effect modules
cargo tarpaulin --lib --out Html

# Check coverage for specific effect
cargo tarpaulin --lib -p symlink
```

Source: [src/git.rs](https://github.com/your-org/twin/blob/main/src/git.rs), [src/config.rs](https://github.com/your-org/twin/blob/main/src/config.rs)
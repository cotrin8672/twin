# Testing Strategy

Twin's testing strategy focuses on verifying both Git worktree operations and their associated effects.

## Testing Philosophy

Since Twin is a Git worktree wrapper with effect management, our testing approach validates:

1. **Primary Operations**: Git worktree creation/removal work correctly
2. **Effect Execution**: Effects are triggered at the right time
3. **Effect Isolation**: Effects can be tested independently
4. **Effect Recovery**: Failed effects don't break primary operations
5. **Cross-Platform Effects**: Effects work consistently across platforms

## Testing Levels

### 1. Unit Tests - Effect Logic
Test individual effect implementations in isolation:
- Symlink effect strategies (Unix vs Windows)
- Hook command parsing and variable substitution
- Configuration effect loading and validation
- Error handling for each effect type

### 2. Integration Tests - Effect Chains
Test how effects work together with Git operations:
- Complete worktree creation with all effects
- Effect execution order and dependencies
- Effect failure handling and recovery
- Platform-specific effect fallbacks

### 3. End-to-End Tests - User Workflows
Test complete user scenarios with effects:
- Create worktree → Apply effects → Verify results
- Remove worktree → Cleanup effects → Verify cleanup
- Effect reporting and user feedback

## Test Organization for Effects

```
tests/
├── common/                      # Shared test utilities
│   └── mod.rs                  # Effect test helpers
├── git_worktree_wrapper_test.rs # Primary operation tests
├── symlink_test.rs             # Symlink effect tests
├── hook_integration_test.rs    # Hook effect tests
├── integration_test.rs         # Effect chain tests
└── e2e_basic.rs               # Complete workflow tests
```

## Testing Effect Behaviors

### Testing Symlink Effects
```rust
#[test]
fn test_symlink_effect_with_fallback() {
    // Arrange: Set up worktree context
    let context = create_test_context();
    
    // Act: Apply symlink effect
    let effect = SymlinkEffect::new(source, target);
    let result = effect.apply(&context);
    
    // Assert: Verify effect outcome
    assert!(result.is_ok());
    if can_create_symlink() {
        assert!(is_symlink(&target));
    } else {
        assert!(is_regular_file(&target));
    }
}
```

### Testing Hook Effects
```rust
#[test]
fn test_hook_effect_variable_substitution() {
    // Test that hook effects properly substitute variables
    let hook = HookEffect::new("echo {branch}");
    let context = WorktreeContext {
        branch_name: "feature".to_string(),
        // ...
    };
    
    let command = hook.prepare(&context);
    assert_eq!(command, "echo feature");
}
```

### Testing Effect Chains
```rust
#[test]
fn test_effect_chain_continues_on_failure() {
    // Verify that effect chain continues even if one effect fails
    let effects = vec![
        Box::new(FailingEffect),
        Box::new(SuccessfulEffect),
    ];
    
    let results = EffectChain::execute(effects, &context);
    assert_eq!(results.len(), 2);
    assert!(!results[0].success);
    assert!(results[1].success);
}
```

## Platform-Specific Effect Testing

### Windows Effect Testing
```rust
#[cfg(windows)]
#[test]
fn test_windows_symlink_fallback_to_copy() {
    // When symlinks aren't available, test copy fallback
    let manager = WindowsSymlinkManager::new();
    manager.set_force_copy_mode(true);
    
    let result = manager.create_link(&source, &target);
    assert!(result.is_ok());
    assert!(is_regular_file(&target));
}
```

### Unix Effect Testing
```rust
#[cfg(unix)]
#[test]
fn test_unix_symlink_creation() {
    let manager = UnixSymlinkManager::new();
    let result = manager.create_link(&source, &target);
    assert!(result.is_ok());
    assert!(is_symlink(&target));
}
```

## Testing Tools

### Mock Effects for Testing
```rust
use mockall::{automock, predicate::*};

#[automock]
trait Effect {
    fn apply(&self, context: &WorktreeContext) -> Result<()>;
}

#[test]
fn test_effect_orchestration() {
    let mut mock = MockEffect::new();
    mock.expect_apply()
        .times(1)
        .returning(|_| Ok(()));
    
    // Test effect orchestration logic
}
```

### Test Fixtures for Effects
```rust
pub struct EffectTestFixture {
    temp_dir: TempDir,
    repo: Repository,
    context: WorktreeContext,
}

impl EffectTestFixture {
    pub fn new() -> Self {
        // Set up test environment for effects
    }
    
    pub fn with_config(mut self, config: Config) -> Self {
        // Configure effects for testing
    }
}
```

## Container-Based Effect Testing

For complex effect scenarios:
```rust
#[test]
fn test_effects_in_container() {
    let docker = testcontainers::clients::Cli::default();
    let container = docker.run(create_test_image());
    
    // Test effects in isolated environment
    container.exec(vec!["twin", "create", "test"]);
    
    // Verify effects were applied
    let symlinks = container.exec(vec!["find", "-type", "l"]);
    assert!(symlinks.contains(expected_symlink));
}
```

## Performance Testing for Effects

```rust
#[test]
fn test_effect_performance() {
    let start = Instant::now();
    
    // Create worktree with many effects
    let effects = generate_many_effects(100);
    EffectChain::execute(effects, &context)?;
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(5), 
            "Effects took too long: {:?}", duration);
}
```

## Test Coverage Goals

- **Primary Operations**: 100% coverage of Git worktree operations
- **Effect Types**: 100% coverage of each effect implementation
- **Effect Combinations**: Key combinations of effects tested
- **Error Paths**: All effect failure modes tested
- **Platform Variations**: Platform-specific effects tested on each OS

## Continuous Integration Testing

```yaml
# .github/workflows/test.yml
name: Test Effects
on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v2
      - name: Test primary operations
        run: cargo test git_worktree
      - name: Test effects
        run: cargo test effect
      - name: Test platform-specific effects
        run: cargo test --features platform-tests
```

Source: [Cargo.toml#L26-37](https://github.com/your-org/twin/blob/main/Cargo.toml#L26-37), [tests/](https://github.com/your-org/twin/blob/main/tests/)
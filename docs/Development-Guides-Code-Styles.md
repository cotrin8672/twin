# Code Style Guide

Consistent code style for Twin's effect-oriented architecture.

## Core Philosophy

Twin is a Git worktree wrapper with effect management. All code should reflect this separation:
- **Primary operations**: Git worktree manipulation
- **Effects**: Everything else (symlinks, hooks, configs)

## Rust Style Guidelines

### General Principles

1. Follow standard Rust conventions
2. Use `cargo fmt` for automatic formatting
3. Use `cargo clippy` for linting
4. Write idiomatic Rust code
5. Effects are first-class citizens in the architecture

### Naming Conventions

```rust
// Modules: snake_case, effect-oriented naming
mod symlink_effect;    // Not just "symlink"
mod hook_effect;       // Not just "hooks"
mod effect_chain;      // Clear purpose

// Types: PascalCase, clear effect relationship
struct SymlinkEffect;      // Not SymlinkManager
struct HookEffect;         // Not HookExecutor
trait Effect;              // Base trait for all effects
struct EffectResult;       // Not just Result
struct EffectChain;        // Not CommandChain

// Functions: snake_case, verb_noun pattern for effects
fn apply_effect() -> Result<EffectResult>;
fn can_apply_effect() -> bool;
fn rollback_effect() -> Result<()>;

// Constants: SCREAMING_SNAKE_CASE
const MAX_EFFECT_RETRIES: u32 = 3;
const DEFAULT_HOOK_TIMEOUT: u64 = 60;
```

### Effect Implementation Pattern

All effects should follow this consistent pattern:

```rust
/// Documentation should explain what effect this provides
pub struct CustomEffect {
    // Effect configuration
    config: EffectConfig,
}

impl CustomEffect {
    /// Creates a new effect instance
    pub fn new(config: EffectConfig) -> Self {
        Self { config }
    }
    
    /// Validates the effect can be applied
    fn validate(&self, context: &WorktreeContext) -> Result<()> {
        // Validation logic
        Ok(())
    }
}

impl Effect for CustomEffect {
    /// Apply the effect to the worktree
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        self.validate(context)?;
        
        // Effect implementation
        
        Ok(EffectResult::success("Effect applied"))
    }
    
    /// Check if the effect can be applied
    fn can_apply(&self, context: &WorktreeContext) -> bool {
        self.validate(context).is_ok()
    }
    
    /// Rollback the effect if possible
    fn rollback(&self, context: &WorktreeContext) -> Result<()> {
        // Rollback implementation
        Ok(())
    }
    
    /// Effect type for reporting
    fn effect_type(&self) -> &str {
        "custom"
    }
}
```

### Error Handling for Effects

```rust
// Effects should use specific error types
pub enum EffectError {
    // Critical - stops all effects
    Critical(String),
    
    // Recoverable - try fallback
    Recoverable { 
        message: String,
        fallback: Option<String>,
    },
    
    // Warning - continue with next effect
    Warning(String),
}

// Good: Clear effect error handling
pub fn apply_effect(context: &WorktreeContext) -> Result<EffectResult> {
    match do_effect_work(context) {
        Ok(result) => Ok(EffectResult::success(result)),
        Err(e) if e.is_recoverable() => {
            warn!("Effect failed, trying fallback: {}", e);
            apply_fallback_effect(context)
        }
        Err(e) => Err(EffectError::Critical(e.to_string())),
    }
}

// Avoid: Generic error handling
pub fn apply_effect(context: &WorktreeContext) -> Result<()> {
    do_effect_work(context)?; // No context about effect failure
    Ok(())
}
```

### Documentation Standards

```rust
/// Creates a symlink effect for the worktree.
///
/// This effect creates symbolic links from source files to the worktree,
/// falling back to file copying on platforms without symlink support.
///
/// # Arguments
///
/// * `source` - Path to the source file (relative to repository root)
/// * `target` - Path to the target file (relative to worktree root)
///
/// # Effects
///
/// - Creates a symbolic link from source to target
/// - Falls back to file copy if symlinks unavailable
/// - Skips if target already exists (when configured)
///
/// # Errors
///
/// Returns `EffectError::Recoverable` if symlink fails but copy might work.
/// Returns `EffectError::Critical` if source doesn't exist.
///
/// # Example
///
/// ```rust
/// let effect = SymlinkEffect::new("config.template", ".config");
/// let result = effect.apply(&context)?;
/// ```
pub fn create_symlink_effect(source: &str, target: &str) -> Box<dyn Effect> {
    Box::new(SymlinkEffect::new(source.into(), target.into()))
}
```

### Module Organization

```rust
// Each effect module should have this structure:

// src/symlink_effect.rs
//! Symlink effect implementation for Twin.
//!
//! This module provides the symlink effect that creates symbolic links
//! as part of worktree creation.

use crate::core::{Effect, EffectResult, WorktreeContext};

// Public API first
pub struct SymlinkEffect { ... }

pub fn create_symlink_effect(...) -> Box<dyn Effect> { ... }

// Trait implementations
impl Effect for SymlinkEffect { ... }

// Private implementation details
fn create_platform_symlink(...) -> Result<()> { ... }

// Platform-specific code
#[cfg(unix)]
mod unix_impl { ... }

#[cfg(windows)]
mod windows_impl { ... }

// Tests at the end
#[cfg(test)]
mod tests { ... }
```

### Testing Effects

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    // Test fixture for effects
    fn setup_effect_test() -> (TempDir, WorktreeContext) {
        let temp = tempdir().unwrap();
        let context = WorktreeContext::test_context(temp.path());
        (temp, context)
    }
    
    #[test]
    fn test_effect_success_case() {
        let (temp, context) = setup_effect_test();
        
        // Arrange
        let effect = CustomEffect::new(test_config());
        
        // Act
        let result = effect.apply(&context);
        
        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
        // Verify effect outcome
    }
    
    #[test]
    fn test_effect_fallback() {
        // Test that effects properly fall back
    }
    
    #[test]
    fn test_effect_idempotency() {
        // Test that effects can be applied multiple times safely
    }
}
```

### Async Effects

For future async effect support:

```rust
// Async effects should be explicit
pub trait AsyncEffect {
    async fn apply(&self, context: &WorktreeContext) -> Result<EffectResult>;
}

// Good: Clear async boundaries
pub async fn apply_async_effects(effects: Vec<Box<dyn AsyncEffect>>) -> Vec<EffectResult> {
    let mut results = Vec::new();
    
    for effect in effects {
        let result = effect.apply(&context).await;
        results.push(result);
    }
    
    results
}

// Consider: Parallel effect execution
pub async fn apply_parallel_effects(effects: Vec<Box<dyn AsyncEffect>>) -> Vec<EffectResult> {
    futures::future::join_all(
        effects.into_iter()
            .map(|effect| effect.apply(&context))
    ).await
}
```

## Project-Specific Conventions

### Effect Chain Building

```rust
// Effects should be built declaratively
pub fn build_effect_chain(config: &Config) -> EffectChain {
    EffectChain::builder()
        .add_effects(create_file_effects(&config.files))
        .add_effects(create_hook_effects(&config.hooks))
        .add_effects(create_custom_effects(&config.custom))
        .build()
}

// Not imperatively
pub fn build_effect_chain(config: &Config) -> EffectChain {
    let mut chain = EffectChain::new();
    for file in &config.files {
        if file.should_apply() {
            chain.add(create_effect(file));
        }
    }
    // More complex logic...
}
```

### Effect Result Reporting

```rust
// Consistent effect result creation
impl EffectResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            effect_type: None,
            duration: None,
        }
    }
    
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self {
            success: true,
            message: format!("Skipped: {}", reason.into()),
            effect_type: None,
            duration: None,
        }
    }
    
    pub fn failed(error: impl std::error::Error) -> Self {
        Self {
            success: false,
            message: error.to_string(),
            effect_type: None,
            duration: None,
        }
    }
}
```

### Configuration Parsing for Effects

```rust
// Effect configuration should be strongly typed
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    Symlink,
    Copy,
    Template,
    #[serde(other)]
    Unknown,
}

// Validate at parse time
impl<'de> Deserialize<'de> for EffectConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawEffectConfig::deserialize(deserializer)?;
        raw.validate()
            .map_err(serde::de::Error::custom)?;
        Ok(raw.into())
    }
}
```

## Code Review Checklist for Effects

When reviewing effect-related code:

- [ ] Effect follows the standard Effect trait pattern
- [ ] Effect has proper error handling with fallbacks
- [ ] Effect is idempotent (can be applied multiple times)
- [ ] Effect has comprehensive tests
- [ ] Effect respects platform differences
- [ ] Effect is documented with examples
- [ ] Effect reports clear success/failure messages
- [ ] Effect validates inputs before applying
- [ ] Effect can be rolled back if applicable
- [ ] Effect execution is logged appropriately

Source: Project convention analysis from [src/](https://github.com/your-org/twin/blob/main/src/)
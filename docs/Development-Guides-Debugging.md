# Debugging Guide

Effective debugging techniques for Twin's effect system.

## Understanding Effect Execution Flow

Before debugging, understand how effects flow through the system:

```
User Command → CLI Parser → Git Operation → Effect Chain → Individual Effects → Results
```

## Logging and Tracing Effects

### Enable Effect Tracing

```bash
# Trace all effect execution
export RUST_LOG=twin=trace
cargo run -- create test-branch

# Trace specific effect module
export RUST_LOG=twin::hooks=trace
cargo run -- create test-branch

# Debug effect chain execution
export RUST_LOG=twin::effect_chain=debug
cargo run -- create test-branch
```

### Adding Debug Output to Effects

```rust
use tracing::{debug, info, warn, error, instrument};

impl Effect for CustomEffect {
    #[instrument(skip(self, context))]
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        debug!("Starting effect application");
        debug!(?self.config, "Effect configuration");
        debug!(?context.worktree_path, "Target path");
        
        // Effect implementation
        match self.do_work() {
            Ok(result) => {
                info!("Effect applied successfully");
                Ok(EffectResult::success(result))
            }
            Err(e) => {
                error!(?e, "Effect failed");
                Err(e)
            }
        }
    }
}
```

## Debugging Common Effect Issues

### Issue: Effect Not Triggering

```rust
// Add debug points to verify effect is in chain
impl EffectChain {
    pub fn execute(&self, context: &WorktreeContext) -> Vec<EffectResult> {
        debug!("Effect chain has {} effects", self.effects.len());
        
        for (i, effect) in self.effects.iter().enumerate() {
            debug!("Executing effect {}: {}", i, effect.effect_type());
            
            if !effect.can_apply(context) {
                debug!("Effect {} cannot be applied, skipping", i);
                continue;
            }
            
            // Execute effect
        }
    }
}
```

### Issue: Symlink Effect Fails

```rust
// Debug symlink creation with detailed diagnostics
impl SymlinkEffect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        let source = context.source_path.join(&self.source);
        let target = context.worktree_path.join(&self.target);
        
        debug!("Symlink effect:");
        debug!("  Source: {:?} (exists: {})", source, source.exists());
        debug!("  Target: {:?}", target);
        debug!("  Target parent exists: {}", target.parent().unwrap().exists());
        
        #[cfg(windows)]
        {
            debug!("  Windows mode: checking permissions");
            debug!("  Developer mode: {}", has_developer_mode());
            debug!("  Is admin: {}", is_admin());
        }
        
        match create_symlink(&source, &target) {
            Ok(_) => {
                info!("Symlink created successfully");
                Ok(EffectResult::success("Symlink created"))
            }
            Err(e) => {
                error!("Symlink creation failed: {}", e);
                debug!("Attempting fallback to copy");
                self.fallback_copy(&source, &target)
            }
        }
    }
}
```

### Issue: Hook Effect Variable Substitution

```rust
// Debug variable substitution in hooks
impl HookContext {
    pub fn substitute_variables(&self, command: &str) -> String {
        debug!("Original command: {}", command);
        debug!("Available variables:");
        debug!("  {{branch}} = {}", self.branch_name);
        debug!("  {{worktree_path}} = {:?}", self.worktree_path);
        debug!("  {{source_path}} = {:?}", self.source_path);
        
        let result = command
            .replace("{branch}", &self.branch_name)
            .replace("{worktree_path}", &self.worktree_path.display().to_string())
            .replace("{source_path}", &self.source_path.display().to_string());
        
        debug!("Substituted command: {}", result);
        result
    }
}
```

## IDE Debugging

### Visual Studio Code

Create `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Twin Create",
            "cargo": {
                "args": ["build", "--bin=twin"],
                "filter": {
                    "name": "twin",
                    "kind": "bin"
                }
            },
            "args": ["create", "test-branch"],
            "cwd": "${workspaceFolder}/test-repo",
            "env": {
                "RUST_LOG": "twin=debug",
                "RUST_BACKTRACE": "1"
            }
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Specific Effect",
            "cargo": {
                "args": ["test", "--lib", "--no-run"],
                "filter": {
                    "name": "twin",
                    "kind": "lib"
                }
            },
            "args": ["test_symlink_effect"],
            "env": {
                "RUST_LOG": "debug"
            }
        }
    ]
}
```

### Setting Breakpoints in Effects

```rust
impl Effect for DebugEffect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        // Set breakpoint here to inspect context
        let source = self.resolve_source(context);
        
        // Set breakpoint here to check resolution
        let target = self.resolve_target(context);
        
        // Set breakpoint here before effect application
        self.do_apply(source, target)
    }
}
```

## Debugging Effect Configuration

### Verify Configuration Loading

```rust
// Add debug output to config loading
impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        debug!("Loading config from: {:?}", path);
        
        let content = std::fs::read_to_string(path)?;
        debug!("Config content:\n{}", content);
        
        let config: Config = toml::from_str(&content)?;
        debug!("Parsed config: {:#?}", config);
        
        // Validate effects
        for file in &config.files {
            debug!("File effect: {} -> {}", file.source, file.target);
        }
        
        if let Some(hooks) = &config.hooks {
            debug!("Hook effects:");
            debug!("  Pre-create: {} hooks", hooks.pre_create.len());
            debug!("  Post-create: {} hooks", hooks.post_create.len());
        }
        
        Ok(config)
    }
}
```

### Test Configuration in Isolation

```bash
# Create test configuration
cat > debug.toml << EOF
worktree_base = "./debug-worktrees"

[[files]]
source = "test.txt"
target = "test.txt"
mapping_type = "symlink"

[hooks]
post_create = [
    { command = "echo 'Effect executed' > effect.log" }
]
EOF

# Test with debug output
RUST_LOG=twin::config=debug cargo run -- create debug --config debug.toml
```

## Debugging Platform-Specific Effects

### Windows Debugging

```rust
#[cfg(windows)]
fn debug_windows_permissions() {
    use windows_sys::Win32::System::SystemInformation::*;
    
    debug!("Windows environment:");
    debug!("  Developer mode: {}", check_developer_mode());
    debug!("  Admin rights: {}", is_elevated());
    debug!("  Symlink privilege: {}", has_symlink_privilege());
    
    // Test symlink creation
    let test_dir = std::env::temp_dir();
    let source = test_dir.join("test_source.txt");
    let target = test_dir.join("test_target.txt");
    
    std::fs::write(&source, "test").unwrap();
    
    match std::os::windows::fs::symlink_file(&source, &target) {
        Ok(_) => debug!("Test symlink created successfully"),
        Err(e) => debug!("Test symlink failed: {} (error code: {:?})", e, e.raw_os_error()),
    }
}
```

### Unix Debugging

```rust
#[cfg(unix)]
fn debug_unix_permissions() {
    use std::os::unix::fs::MetadataExt;
    
    debug!("Unix environment:");
    debug!("  UID: {}", std::process::id());
    debug!("  Effective UID: {}", unsafe { libc::geteuid() });
    
    // Check file permissions
    let meta = std::fs::metadata(".").unwrap();
    debug!("  Current dir mode: {:o}", meta.mode());
    debug!("  Current dir owner: {}", meta.uid());
}
```

## Performance Debugging

### Profile Effect Execution

```rust
use std::time::Instant;

impl Effect for TimedEffect {
    fn apply(&self, context: &WorktreeContext) -> Result<EffectResult> {
        let start = Instant::now();
        
        let result = self.do_apply(context)?;
        
        let duration = start.elapsed();
        debug!("Effect {} took {:?}", self.effect_type(), duration);
        
        if duration > Duration::from_secs(1) {
            warn!("Slow effect detected: {} took {:?}", self.effect_type(), duration);
        }
        
        Ok(result)
    }
}
```

### Memory Debugging

```bash
# Check for memory leaks with valgrind (Linux)
valgrind --leak-check=full --show-leak-kinds=all \
    target/debug/twin create test-branch

# Use heaptrack for detailed memory profiling
heaptrack target/debug/twin create test-branch
heaptrack --analyze heaptrack.twin.*.gz
```

## Debugging Effect Chains

```rust
// Debug effect chain execution
pub struct DebugEffectChain {
    effects: Vec<Box<dyn Effect>>,
}

impl DebugEffectChain {
    pub fn execute(&self, context: &WorktreeContext) -> Vec<EffectResult> {
        let mut results = Vec::new();
        
        for (i, effect) in self.effects.iter().enumerate() {
            debug!("=" * 50);
            debug!("Effect {}/{}: {}", i + 1, self.effects.len(), effect.effect_type());
            debug!("Can apply: {}", effect.can_apply(context));
            
            let start = Instant::now();
            let result = effect.apply(context);
            let duration = start.elapsed();
            
            match &result {
                Ok(r) => debug!("✓ Success: {} ({:?})", r.message, duration),
                Err(e) => debug!("✗ Failed: {} ({:?})", e, duration),
            }
            
            results.push(result);
            debug!("=" * 50);
        }
        
        debug!("Effect chain complete: {}/{} succeeded", 
               results.iter().filter(|r| r.is_ok()).count(),
               results.len());
        
        results
    }
}
```

## Common Debug Commands

```bash
# Debug configuration loading
RUST_LOG=twin::config=trace cargo run -- config show

# Debug Git operations
RUST_LOG=twin::git=debug cargo run -- create test

# Debug all effects
RUST_LOG=twin::symlink=debug,twin::hooks=debug cargo run -- create test

# Full debug with backtrace
RUST_LOG=debug RUST_BACKTRACE=full cargo run -- create test

# Debug specific test
RUST_LOG=debug cargo test test_effect_chain -- --nocapture
```

Source: [src/main.rs#L21-39](https://github.com/your-org/twin/blob/main/src/main.rs#L21-39)
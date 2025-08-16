# Configuration Guide

Detailed configuration options for Twin's effect system in deployment environments.

## Configuration Philosophy

Twin uses declarative configuration to define effects. The configuration describes **what effects to apply** when worktrees are created or removed, not **how** to manage worktrees (that's Twin's job).

## Configuration File Locations

Twin searches for effect configurations in order:

1. **Command-line specified**: `--config path/to/effects.toml`
2. **Project root**: `./twin.toml` (default for project-specific effects)
3. **User config**: `~/.config/twin/config.toml` (user's default effects)
4. **System config**: `/etc/twin/config.toml` (system-wide effects)

## Complete Effect Configuration Reference

```toml
# twin.toml - Complete effect configuration

# Primary configuration (Git worktree behavior)
worktree_base = "../workspaces"  # Base directory for worktrees

# ============================================================
# FILE EFFECTS - Applied to files in the worktree
# ============================================================

[[files]]
# Source file (relative to repository root)
source = "configs/dev.env"

# Target location in worktree (relative to worktree root)
target = ".env"

# Effect type (how to handle the file)
# - "symlink": Create symbolic link (effect)
# - "copy": Copy the file (effect)
# - "template": Process as template (future effect)
mapping_type = "symlink"

# Effect behavior options
skip_if_exists = true  # Don't override existing files
create_parent = true    # Create parent directories (effect)

# Effect metadata
description = "Development environment configuration"
required = false        # Effect failure doesn't stop worktree creation

# Platform-specific effect behavior
[files.platform]
windows = { mapping_type = "copy" }  # Different effect on Windows
unix = { mapping_type = "symlink" }

# ============================================================
# HOOK EFFECTS - Commands executed at lifecycle points
# ============================================================

[hooks]
# Pre-creation effects (before worktree exists)
pre_create = [
    {
        command = "echo 'Preparing worktree for {branch}'",
        continue_on_error = false,  # Critical effect
        timeout = 60,               # Effect timeout in seconds
        description = "Announce worktree creation"
    }
]

# Post-creation effects (after worktree created)
post_create = [
    {
        command = "cd {worktree_path} && npm install",
        continue_on_error = true,   # Non-critical effect
        timeout = 300,
        description = "Install dependencies",
        platforms = ["unix", "darwin"]  # Platform-specific effect
    },
    {
        command = "cd {worktree_path} && pip install -r requirements.txt",
        continue_on_error = true,
        timeout = 300,
        description = "Install Python dependencies",
        condition = "exists:requirements.txt"  # Conditional effect
    }
]

# Pre-removal effects (before worktree removal)
pre_remove = [
    {
        command = "cd {worktree_path} && npm run cleanup",
        continue_on_error = true,
        timeout = 60,
        description = "Cleanup before removal"
    }
]

# Post-removal effects (after worktree removed)
post_remove = [
    {
        command = "echo 'Worktree {branch} removed'",
        continue_on_error = true,
        description = "Log removal"
    }
]

# ============================================================
# ADVANCED EFFECT CONFIGURATION
# ============================================================

# Effect execution settings
[effects]
parallel = false          # Execute effects in parallel (future)
max_parallel = 4         # Max parallel effects
stop_on_error = false    # Continue on effect failure
log_level = "info"       # Effect logging level

# Effect retry configuration
[effects.retry]
max_attempts = 3
delay_seconds = 1
exponential_backoff = true

# Platform-specific effect defaults
[effects.platform.windows]
default_mapping = "copy"
symlink_fallback = true

[effects.platform.unix]
default_mapping = "symlink"
preserve_permissions = true
```

## Environment Variables for Effects

Twin effects can be controlled via environment variables:

```bash
# Effect execution control
export TWIN_SKIP_EFFECTS=1        # Skip all effects
export TWIN_SKIP_HOOKS=1          # Skip hook effects only
export TWIN_SKIP_SYMLINKS=1       # Skip symlink effects only

# Effect debugging
export TWIN_EFFECT_DEBUG=1        # Debug effect execution
export TWIN_EFFECT_TRACE=1        # Trace all effects
export RUST_LOG=twin::effect=debug # Detailed effect logging

# Effect behavior
export TWIN_EFFECT_TIMEOUT=600    # Global effect timeout
export TWIN_FORCE_COPY=1          # Force copy instead of symlink
export TWIN_PARALLEL_EFFECTS=1    # Enable parallel effects (future)

# Configuration override
export TWIN_CONFIG=/path/to/effects.toml
export TWIN_NO_USER_CONFIG=1      # Skip user config
export TWIN_NO_SYSTEM_CONFIG=1    # Skip system config
```

## Platform-Specific Effect Configuration

### Windows Effect Configuration
```toml
# Windows-specific effects
[[files]]
source = "scripts/setup.ps1"
target = "setup.ps1"
mapping_type = "copy"  # Always copy on Windows

[hooks.post_create]
# PowerShell effect
command = "powershell.exe -ExecutionPolicy Bypass -File {worktree_path}\\setup.ps1"
platforms = ["windows"]

# Effect with Windows paths
[[files]]
source = "config\\windows.ini"
target = "config.ini"
platform = "windows"  # Only apply on Windows
```

### Unix/Linux/macOS Effect Configuration
```toml
# Unix-specific effects
[[files]]
source = "scripts/setup.sh"
target = "setup.sh"
mapping_type = "symlink"
permissions = "755"  # Unix permission effect

[hooks.post_create]
# Shell effect
command = "cd {worktree_path} && chmod +x setup.sh && ./setup.sh"
platforms = ["unix", "darwin"]
shell = "/bin/bash"  # Specific shell for effect
```

## Effect Profiles

Define reusable effect profiles:

```toml
# Define effect profiles
[profiles.nodejs]
files = [
    { source = ".nvmrc", target = ".nvmrc", mapping_type = "symlink" },
    { source = ".npmrc", target = ".npmrc", mapping_type = "copy" }
]
hooks.post_create = [
    { command = "nvm use", continue_on_error = true },
    { command = "npm install", continue_on_error = false }
]

[profiles.python]
files = [
    { source = ".python-version", target = ".python-version" }
]
hooks.post_create = [
    { command = "pyenv install", continue_on_error = true },
    { command = "pip install -r requirements.txt" }
]

# Use profiles
[config]
use_profiles = ["nodejs", "python"]
```

## Conditional Effects

Effects can be conditionally applied:

```toml
[[files]]
source = ".env.development"
target = ".env"
condition = "!exists:.env"  # Only if .env doesn't exist

[hooks.post_create]
command = "docker-compose up -d"
condition = "exists:docker-compose.yml"  # Only if file exists

[[files]]
source = "config.{env}.toml"
target = "config.toml"
env_var = "TWIN_ENV"  # Use environment variable
```

## Effect Validation

Twin validates effect configurations:

```bash
# Validate configuration
twin config validate

# Test effect configuration
twin config test --dry-run

# Show effective configuration (after merging)
twin config show --effective
```

## Example Configurations

### Minimal Configuration (basic effects)
```toml
worktree_base = "../"

[[files]]
source = "README.md"
target = "README.md"
mapping_type = "symlink"
```

### Development Environment (full effects)
```toml
worktree_base = "../dev"

# Shared configuration files
[[files]]
source = ".env.development"
target = ".env"
mapping_type = "copy"
skip_if_exists = true

[[files]]
source = "docker-compose.override.yml"
target = "docker-compose.override.yml"
mapping_type = "symlink"

# Development setup hooks
[hooks.post_create]
commands = [
    { command = "npm install", timeout = 300 },
    { command = "npm run setup:dev" },
    { command = "docker-compose up -d", continue_on_error = true }
]

[hooks.pre_remove]
commands = [
    { command = "docker-compose down", continue_on_error = true }
]
```

### CI/CD Configuration (automated effects)
```toml
worktree_base = "/tmp/ci-worktrees"

# CI-specific effects
[effects]
stop_on_error = true  # Fail fast in CI
log_level = "debug"

[[files]]
source = "ci/config.yml"
target = ".ci/config.yml"
required = true  # Must succeed

[hooks.post_create]
commands = [
    { command = "ci-validator check", timeout = 60 },
    { command = "npm ci", timeout = 300 },
    { command = "npm test", timeout = 600 }
]
```

### Multi-Project Configuration (complex effects)
```toml
# Shared workspace configuration
worktree_base = "../workspaces/{project}"

# Project-specific effects
[projects.frontend]
files = [
    { source = "frontend/.env", target = ".env" }
]
hooks.post_create = [
    { command = "cd frontend && npm install" }
]

[projects.backend]
files = [
    { source = "backend/config.toml", target = "config.toml" }
]
hooks.post_create = [
    { command = "cd backend && cargo build" }
]
```

## Security Considerations

```toml
# Security settings for effects
[security]
# Restrict hook commands
allowed_commands = ["npm", "cargo", "make", "docker"]
forbidden_patterns = ["rm -rf", "sudo", "curl | sh"]

# Sandbox effects (future feature)
sandbox_hooks = true
sandbox_user = "twin-effects"

# Audit effect execution
audit_log = "/var/log/twin-effects.log"
```

Source: [README.md#L98-163](https://github.com/your-org/twin/blob/main/README.md#L98-163), [src/core/types.rs#L52-112](https://github.com/your-org/twin/blob/main/src/core/types.rs#L52-112)
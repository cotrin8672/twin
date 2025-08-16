# Architecture Overview

Twin follows a modular architecture designed for extensibility and cross-platform compatibility.

## High-Level Architecture

```mermaid
graph TB
    CLI[CLI Layer] --> Core[Core Business Logic]
    Core --> Git[Git Management]
    Core --> Symlink[Symlink Management]
    Core --> Config[Configuration]
    Core --> Hooks[Hook System]
    
    Git --> git2[libgit2]
    Symlink --> OS[OS-specific Implementations]
    Config --> TOML[TOML Parser]
    
    subgraph Platform Layer
        OS --> Unix[Unix/Linux/macOS]
        OS --> Windows[Windows]
    end
```

## Component Responsibilities

### CLI Layer (`src/cli/`)
- Command parsing and validation
- Output formatting (table, JSON, simple)
- User interaction handling

### Core Module (`src/core/`)
- Central types and data structures
- Error handling and propagation
- Business logic coordination

### Git Management (`src/git.rs`)
- Worktree operations (create, list, remove)
- Branch management
- Repository validation

### Symlink Management (`src/symlink.rs`)
- Platform-specific symlink creation
- Permission handling
- Fallback strategies (copy mode)

### Configuration (`src/config.rs`)
- TOML file parsing
- Configuration merging
- Default values

### Hook System (`src/hooks.rs`)
- Lifecycle event handling
- Command execution
- Variable substitution

Source: [src/main.rs#L1-16](https://github.com/your-org/twin/blob/main/src/main.rs#L1-16)
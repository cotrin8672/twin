# Technology Stack

Twin is built with modern Rust tooling and carefully selected dependencies optimized for its role as a Git worktree wrapper with effect management.

## Programming Language

**Rust** (Edition 2024)
- Memory safety without garbage collection
- Cross-platform compilation
- Excellent error handling with Result types
- Strong type system for effect management

Source: [Cargo.toml#L4](https://github.com/your-org/twin/blob/main/Cargo.toml#L4)

## Core Dependencies

### Git Worktree Operations (Primary)
- **git2** (0.20.2) - libgit2 bindings for Rust
  - Worktree creation and management
  - Branch operations
  - Repository state validation

### Effect Management

#### Configuration Effects
- **toml** (0.9.5) - TOML file parsing for effect definitions
- **serde** (1.0.219) - Serialization framework for effect configurations
- **serde_json** (1.0.142) - JSON output for effect results

#### File System Effects
- **directories** (6.0.0) - Platform-specific directory paths
- **tempfile** (3.20.0) - Temporary file/directory for effect isolation
- **which** (8.0.0) - Command resolution for hook effects

### Command Interface
- **clap** (4.5.44) - Command-line argument parsing
  - Subcommand structure for worktree operations
  - Environment variable integration
  - Auto-generated help

### Error Handling for Effects
- **anyhow** (1.0.99) - Flexible error handling for effect chains
- **thiserror** (2.0.14) - Custom error types for effect failures

### Async Effect Execution
- **tokio** (1.47.1) - Async runtime for parallel effect processing
  - Concurrent hook execution
  - Async file operations
  - Process spawning for effects

### Future: Interactive Effects
- **ratatui** (0.29.0) - Terminal UI for effect visualization
- **crossterm** (0.29.0) - Cross-platform terminal manipulation
- **notify** (8.2.0) - File system monitoring for reactive effects

### Effect Logging & Tracing
- **tracing** (0.1.41) - Structured logging for effect execution
- **tracing-subscriber** (0.3.19) - Effect execution trace configuration
- **log** (0.4.27) - Legacy logging support

### Utilities
- **chrono** (0.4.41) - Timestamp for effect execution tracking

## Development Dependencies

### Testing Effect Behaviors
- **mockall** (0.13.1) - Mock effect implementations
- **pretty_assertions** (1.4.1) - Clear effect assertion output
- **tempfile** (3.20.0) - Isolated effect testing

### Container-Based Effect Testing
- **testcontainers** (0.23.1) - Test effects in containers
- **bollard** (0.18.1) - Docker API for effect environment setup
- **futures-util** (0.3) - Async effect test utilities
- **uuid** (1.11) - Unique identifiers for effect test isolation
- **tar** (0.4) - Archive handling for containerized effects

### Build & Development
- **cargo-husky** (1.5.0) - Git hooks for development
- **paste** (1.0) - Macro utilities for effect definitions

## Architecture Philosophy

The technology stack is chosen to support Twin's core concept:
1. **Git Worktree Wrapper**: git2 provides low-level Git operations
2. **Effect Management**: Async runtime enables parallel effect execution
3. **Cross-Platform Effects**: Platform abstractions ensure consistent behavior
4. **Declarative Configuration**: TOML enables clear effect definitions

Source: [Cargo.toml#L6-37](https://github.com/your-org/twin/blob/main/Cargo.toml#L6-37)
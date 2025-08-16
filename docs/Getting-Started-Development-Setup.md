# Development Setup

This guide walks you through setting up your development environment for Twin.

## Prerequisites

### Required Software

1. **Rust** (1.70.0 or later)
   ```bash
   # Install via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Git** (2.15.0 or later - worktree support required)
   ```bash
   git --version
   ```

3. **Platform-Specific Requirements**

   **Windows**:
   - Enable Developer Mode or run as Administrator for symlink support
   - Visual Studio Build Tools (for compilation)

   **macOS/Linux**:
   - Standard build tools (gcc/clang)

## Setting Up the Development Environment

1. **Clone the Repository**
   ```bash
   git clone https://github.com/your-org/twin.git
   cd twin
   ```

2. **Install Dependencies**
   ```bash
   cargo fetch
   ```

3. **Build the Project**
   ```bash
   # Debug build
   cargo build
   
   # Release build
   cargo build --release
   ```

4. **Run Tests**
   ```bash
   # All tests
   cargo test
   
   # Unit tests only
   cargo test --lib
   
   # Integration tests
   cargo test --test '*'
   ```

## IDE Setup

### Visual Studio Code
1. Install Rust Analyzer extension
2. Install TOML Language Support extension
3. Recommended settings (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```

### RustRover/IntelliJ IDEA
1. Install Rust plugin
2. Open project and let IDE index

## Development Tools

### Linting and Formatting
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

### Debugging
The project uses `tracing` for structured logging:
```bash
# Enable debug logging
export TWIN_DEBUG=1
cargo run -- create test-branch

# Or use RUST_LOG
export RUST_LOG=twin=debug
cargo run -- list
```

Source: [Cargo.toml#L1-37](https://github.com/your-org/twin/blob/main/Cargo.toml#L1-37)
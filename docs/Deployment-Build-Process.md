# Build Process

Building Twin for different platforms and distributions, optimized for its role as a Git worktree wrapper with effect management.

## Local Development Build

### Debug Build (with full effect debugging)
```bash
# Standard debug build with effect tracing
cargo build
# Binary at: ./target/debug/twin

# Run with effect debugging
RUST_LOG=twin::effect=debug ./target/debug/twin create test
```

### Release Build (optimized for effects)
```bash
# Optimized build with effect performance
cargo build --release
# Binary at: ./target/release/twin

# Smaller binary with stripped symbols
cargo build --release --features minimal
```

## Cross-Platform Effect Support

Twin's effects must work consistently across platforms:

### Building for Windows
```bash
# From Unix/Linux
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# Verify Windows-specific effects
cargo test --target x86_64-pc-windows-gnu symlink_fallback
```

### Building for Linux
```bash
# From macOS/Windows
rustup target add x86_64-unknown-linux-gnu

# Using cross for easier compilation
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu

# Test Linux-specific effects
cross test --target x86_64-unknown-linux-gnu unix_symlink
```

### Building for macOS
```bash
# From Linux/Windows (requires macOS SDK)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# For Apple Silicon
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

## Build Optimization for Effects

### Optimize for Effect Performance
```toml
# Cargo.toml
[profile.release]
opt-level = 3           # Maximum performance for effects
lto = "fat"            # Link-time optimization
codegen-units = 1      # Better optimization
panic = "abort"        # Smaller binary
strip = "symbols"      # Remove debug symbols

[profile.release.package.git2]
opt-level = 3          # Optimize Git operations

[profile.release.package."*"]
opt-level = 2          # Optimize dependencies
```

### Optimize for Binary Size
```toml
[profile.minimal]
inherits = "release"
opt-level = "z"        # Optimize for size
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

## Feature Flags for Effects

```toml
# Cargo.toml
[features]
default = ["all-effects"]
all-effects = ["symlink-effects", "hook-effects", "template-effects"]
symlink-effects = []
hook-effects = []
template-effects = []  # Future feature
minimal = []          # Minimal build without optional effects

# Platform-specific effect features
windows-effects = ["windows", "winapi"]
unix-effects = []
```

Build with specific effects:
```bash
# Only core + symlink effects
cargo build --release --no-default-features --features symlink-effects

# Minimal build (Git operations only)
cargo build --release --no-default-features --features minimal
```

## CI/CD Build Pipeline

### GitHub Actions for Multi-Platform Effects

```yaml
name: Build and Test Effects

on:
  push:
    tags:
      - 'v*'
  pull_request:

jobs:
  test-effects:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            effect-tests: unix-effects
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            effect-tests: windows-effects
          - os: macos-latest
            target: x86_64-apple-darwin
            effect-tests: unix-effects
          - os: macos-latest
            target: aarch64-apple-darwin
            effect-tests: unix-effects
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Test Platform Effects
        run: cargo test --features ${{ matrix.effect-tests }}
      
      - name: Build with Effects
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Test Effect Integration
        run: |
          ./target/${{ matrix.target }}/release/twin create test-branch
          ./target/${{ matrix.target }}/release/twin list
          ./target/${{ matrix.target }}/release/twin remove test-branch
      
      - name: Upload Binary
        uses: actions/upload-artifact@v3
        with:
          name: twin-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/twin*
```

## Creating Distributions

### Homebrew Formula (macOS/Linux)
```ruby
class Twin < Formula
  desc "Git worktree wrapper with effect management"
  homepage "https://github.com/your-org/twin"
  version "0.1.0"
  
  if OS.mac?
    url "https://github.com/your-org/twin/releases/download/v0.1.0/twin-darwin-x64.tar.gz"
    sha256 "..."
  elsif OS.linux?
    url "https://github.com/your-org/twin/releases/download/v0.1.0/twin-linux-x64.tar.gz"
    sha256 "..."
  end
  
  def install
    bin.install "twin"
  end
  
  test do
    system "#{bin}/twin", "--version"
  end
end
```

### Cargo Package

```toml
# Cargo.toml
[package]
name = "twin"
version = "0.1.0"
authors = ["Your Name <email@example.com>"]
edition = "2024"
description = "Git worktree wrapper with effect management"
repository = "https://github.com/your-org/twin"
license = "MIT"
keywords = ["git", "worktree", "effects", "development", "tools"]
categories = ["command-line-utilities", "development-tools"]

[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.tar.gz"
```

Publish to crates.io:
```bash
cargo publish --dry-run
cargo publish
```

### Debian/Ubuntu Package

Create `debian/control`:
```
Package: twin
Version: 0.1.0
Architecture: amd64
Maintainer: Your Name <email@example.com>
Description: Git worktree wrapper with effect management
 Twin simplifies Git worktree operations and manages
 their side effects including symlinks, hooks, and configs.
Depends: git (>= 2.15.0)
```

Build:
```bash
cargo install cargo-deb
cargo deb
# Output: target/debian/twin_0.1.0_amd64.deb
```

### Windows Installer

Using WiX Toolset:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="Twin" Version="0.1.0" Manufacturer="Your Org">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="Twin">
          <Component Id="TwinExecutable">
            <File Source="target\release\twin.exe" />
          </Component>
        </Directory>
      </Directory>
    </Directory>
    
    <Feature Id="ProductFeature">
      <ComponentRef Id="TwinExecutable" />
    </Feature>
    
    <!-- Add to PATH -->
    <Environment Id="PATH" Name="PATH" Value="[INSTALLFOLDER]" 
                 Permanent="no" Part="last" Action="set" System="yes" />
  </Product>
</Wix>
```

## Docker Image for Effects Testing

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/twin /usr/local/bin/twin

# Test effects work in container
RUN twin --version && \
    git config --global user.email "test@example.com" && \
    git config --global user.name "Test User"

ENTRYPOINT ["twin"]
```

## Verification Script

```bash
#!/bin/bash
# verify-build.sh - Verify Twin build with effects

set -e

echo "Testing Twin build with effects..."

# Create test repository
temp_dir=$(mktemp -d)
cd "$temp_dir"
git init
echo "# Test" > README.md
git add README.md
git commit -m "Initial commit"

# Create effect configuration
cat > twin.toml << EOF
[[files]]
source = "README.md"
target = "README.md"
mapping_type = "symlink"

[hooks]
post_create = [
    { command = "echo 'Effect test' > effect.txt" }
]
EOF

# Test Twin with effects
twin create test-branch
if [ ! -f "../test-branch/effect.txt" ]; then
    echo "ERROR: Hook effect failed"
    exit 1
fi

if [ ! -e "../test-branch/README.md" ]; then
    echo "ERROR: Symlink effect failed"
    exit 1
fi

echo "✓ All effects working correctly"

# Cleanup
twin remove test-branch
cd -
rm -rf "$temp_dir"
```

## Build Matrix

| Platform | Architecture | Effects Support | Binary Size |
|----------|-------------|-----------------|-------------|
| Linux | x86_64 | Full | ~8 MB |
| Linux | aarch64 | Full | ~8 MB |
| macOS | x86_64 | Full | ~9 MB |
| macOS | aarch64 | Full | ~9 MB |
| Windows | x86_64 | Full with fallbacks | ~10 MB |
| Windows | aarch64 | Full with fallbacks | ~10 MB |

Source: [Cargo.toml](https://github.com/your-org/twin/blob/main/Cargo.toml)
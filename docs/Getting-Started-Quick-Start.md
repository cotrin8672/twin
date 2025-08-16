# Quick Start Guide

Get Twin up and running in 5 minutes.

## Installation

### From Source
```bash
# Clone and build
git clone https://github.com/your-org/twin.git
cd twin
cargo install --path .

# Verify installation
twin --version
```

### From Cargo (Future)
```bash
cargo install twin
```

## Basic Usage

### 1. Initialize Your Project
Navigate to your Git repository:
```bash
cd your-project
```

### 2. Create Your First Environment
```bash
# Create a new worktree environment
twin create feature-authentication

# Output shows the created path
# ../feature-authentication
```

### 3. List Environments
```bash
twin list

# Output:
# ┌──────────────────────┬─────────────────────────┬────────┐
# │ Branch               │ Path                    │ Prunable│
# ├──────────────────────┼─────────────────────────┼────────┤
# │ feature-authentication│ ../feature-authentication│ false  │
# └──────────────────────┴─────────────────────────┴────────┘
```

### 4. Navigate to the Environment
```bash
# Get the cd command
twin create another-feature --cd-command
# Output: cd "../another-feature"

# Or just get the path
twin create test-branch --print-path
# Output: ../test-branch
```

### 5. Remove an Environment
```bash
twin remove feature-authentication
```

## Configuration Example

Create a `twin.toml` file in your project root:
```toml
# Set base directory for worktrees
worktree_base = "../workspaces"

# Define file mappings
[[files]]
source = ".env.template"
target = ".env"
mapping_type = "copy"
skip_if_exists = true
description = "Environment variables"

[[files]]
source = ".claude/config.json"
target = ".claude/config.json"
mapping_type = "symlink"
description = "Claude configuration"
```

Source: [README.md#L27-67](https://github.com/your-org/twin/blob/main/README.md#L27-67)
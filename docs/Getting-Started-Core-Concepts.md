# Core Concepts

Understanding these core concepts will help you effectively use Twin.

## Git Worktrees

Git worktrees allow you to have multiple working directories attached to a single repository. Each worktree:
- Has its own working directory
- Can check out a different branch
- Shares the same Git history and objects
- Reduces disk usage compared to full clones

Twin wraps Git's worktree functionality to provide:
- Automatic branch creation
- Consistent directory structure
- Cleanup and management utilities

## Symbolic Links

Symbolic links (symlinks) are references to files or directories that act as shortcuts. Twin uses them to:
- Share configuration files across environments
- Maintain consistency without duplication
- Support platform-specific implementations

### Platform Differences

**Unix/Linux/macOS**:
- Native symlink support
- No special permissions required

**Windows**:
- Requires Developer Mode or Administrator privileges
- Fallback to file copying if symlinks unavailable
- Automatic detection of available strategies

## File Mappings

File mappings define relationships between source and target files:

```toml
[[files]]
source = "path/to/source"      # Original file
target = "path/in/worktree"    # Destination in worktree
mapping_type = "symlink"       # How to create the link
skip_if_exists = true          # Don't overwrite existing
description = "Purpose"        # Human-readable description
```

### Mapping Types
- **symlink**: Create symbolic link (default)
- **copy**: Copy the file
- **template**: Process as template (planned feature)

## Hooks

Hooks are commands executed at specific lifecycle points:

- **pre_create**: Before worktree creation
- **post_create**: After successful creation
- **pre_remove**: Before worktree removal
- **post_remove**: After successful removal

Hook commands support variable substitution:
- `{branch}`: Branch name
- `{worktree_path}`: Full path to worktree
- `{source_path}`: Repository root path

Source: [README.md#L6-15](https://github.com/your-org/twin/blob/main/README.md#L6-15), [src/core/types.rs#L167-177](https://github.com/your-org/twin/blob/main/src/core/types.rs#L167-177)
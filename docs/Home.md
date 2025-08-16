# Twin - Git Worktree Manager

**English** | [[日本語|JA-Home]]

**A Git worktree wrapper that simplifies worktree operations and manages their side effects**

## Project Overview

Twin is a command-line tool written in Rust that acts as an intelligent wrapper around Git's worktree functionality. Its core philosophy is to make Git worktrees easy to use while automatically managing the side effects (effects) that come with creating and maintaining multiple working directories.

When you create a new worktree, Twin not only handles the Git operations but also manages the necessary effects:
- **Effect Management**: Automatically handles symlinks, file mappings, and hooks as effects of worktree operations
- **Consistency**: Ensures each worktree has the proper configuration and setup
- **Simplicity**: Provides a simple interface to complex Git worktree operations

### Key Features
- 🌲 **Git Worktree Wrapper** - Simplified interface for Git worktree operations
- 🎯 **Effect Management** - Automatic handling of worktree-related side effects
- 🔗 **Symlink Effects** - Create symlinks as an effect of worktree creation
- 🪝 **Hook Effects** - Execute setup commands as effects of worktree lifecycle
- 🖥️ **Cross-Platform** - Consistent behavior across Windows, macOS, and Linux
- ⚙️ **Declarative Configuration** - Define effects in TOML configuration

## Quick Navigation

### Getting Started
- [Development Setup](Getting-Started-Development-Setup)
- [Quick Start Guide](Getting-Started-Quick-Start)
- [Architecture Overview](Getting-Started-Architecture-Overview)
- [Core Concepts](Getting-Started-Core-Concepts)

### Technical Documentation
- [Technology Stack](Architecture-Technology-Stack)
- [System Design](Architecture-System-Design)
- [Module Structure](Architecture-Module-Structure)
- [Data Flow](Architecture-Data-Flow)

### Development Guides
- [Local Development](Development-Guides-Local-Development)
- [Debugging](Development-Guides-Debugging)
- [Code Styles](Development-Guides-Code-Styles)

### Testing
- [Testing Strategy](Testing-Testing-Strategy)
- [Unit Testing](Testing-Unit-Testing)
- [Integration Testing](Testing-Integration-Testing)
- [E2E Testing](Testing-E2E-Testing)

### Deployment
- [Build Process](Deployment-Build-Process)
- [Configuration](Deployment-Configuration)

## Developer Resources

- **Repository**: [GitHub](https://github.com/your-org/twin)
- **Issue Tracker**: [GitHub Issues](https://github.com/your-org/twin/issues)
- **Package**: [crates.io](https://crates.io/crates/twin) (pending)
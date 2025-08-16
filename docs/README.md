# Twin Documentation

Welcome to the Twin documentation. Twin is a Git worktree wrapper that simplifies worktree operations and manages their side effects.

## Documentation Structure

This documentation is organized for use as a GitHub Wiki. Each file represents a Wiki page following the naming convention:
- English: `Category-SubCategory-PageName.md`
- Japanese: `JA-Category-SubCategory-PageName.md`

## Available Documentation

### 🏠 Home Pages
- [Home](Home.md) - Project overview and navigation
- [JA-Home](JA-Home.md) - プロジェクト概要とナビゲーション（日本語）

### 🚀 Getting Started
- [Development Setup](Getting-Started-Development-Setup.md) - Set up your development environment
- [Quick Start](Getting-Started-Quick-Start.md) - Get running in 5 minutes
- [Architecture Overview](Getting-Started-Architecture-Overview.md) - High-level system design
- [Core Concepts](Getting-Started-Core-Concepts.md) - Essential concepts

Japanese versions:
- [JA-開発環境のセットアップ](JA-Getting-Started-Development-Setup.md)
- [JA-クイックスタート](JA-Getting-Started-Quick-Start.md)
- [JA-アーキテクチャ概要](JA-Getting-Started-Architecture-Overview.md)
- [JA-コア概念](JA-Getting-Started-Core-Concepts.md)

### 🏗️ Architecture
- [Technology Stack](Architecture-Technology-Stack.md) - Languages, frameworks, and tools
- [System Design](Architecture-System-Design.md) - Design patterns and principles
- [Module Structure](Architecture-Module-Structure.md) - Code organization
- [Data Flow](Architecture-Data-Flow.md) - How data moves through the system

### 🧪 Testing
- Testing Strategy - Overall testing approach
- Unit Testing - Writing and running unit tests
- Integration Testing - Integration test guidelines
- E2E Testing - End-to-end testing

### 💻 Development Guides
- Local Development - Running locally
- Debugging - Debugging techniques
- Code Styles - Coding conventions

### 📦 Deployment
- Build Process - Building for different platforms
- Configuration - Environment configuration

## Core Concept

Twin's fundamental concept is that it's a **Git worktree wrapper with effect management**:

1. **Primary Function**: Simplify Git worktree operations
2. **Effect Management**: Automatically handle side effects that come with worktree operations
3. **Effect Types**:
   - Symlink creation (file sharing between worktrees)
   - Hook execution (setup commands)
   - File mappings (configuration management)

## Contributing to Documentation

When adding new documentation:
1. Follow the naming convention for Wiki compatibility
2. Add language switcher links for multilingual pages
3. Include source references using GitHub permalinks
4. Use Mermaid syntax for diagrams
5. Verify technical accuracy against the codebase

## Quick Links

- [Repository](https://github.com/your-org/twin)
- [Issues](https://github.com/your-org/twin/issues)
- [Wiki](https://github.com/your-org/twin/wiki)
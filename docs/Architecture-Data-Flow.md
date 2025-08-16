# Data Flow

Understanding how data flows through Twin reveals its effect-oriented architecture.

## Primary Operation Flow with Effects

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Config
    participant GitManager
    participant EffectChain
    participant Symlink
    participant Hooks
    
    User->>CLI: twin create feature-branch
    CLI->>Config: Load effect definitions
    Config-->>CLI: Effect configuration
    
    CLI->>EffectChain: Initialize pre-effects
    EffectChain->>Hooks: Execute pre_create hooks
    Hooks-->>EffectChain: Hook results
    
    CLI->>GitManager: Create worktree (PRIMARY)
    GitManager-->>CLI: Worktree context
    
    CLI->>EffectChain: Apply post-creation effects
    EffectChain->>Symlink: Create symlink effects
    Symlink-->>EffectChain: Symlink results
    
    EffectChain->>Hooks: Execute post_create hooks
    Hooks-->>EffectChain: Hook results
    
    EffectChain-->>CLI: All effect results
    CLI->>User: Display operation + effects
```

## Effect Configuration Flow

```mermaid
graph LR
    Start[Start] --> ConfigFile{twin.toml exists?}
    ConfigFile -->|Yes| LoadEffects[Load effect definitions]
    ConfigFile -->|No| Defaults[Use default effects]
    
    LoadEffects --> ParseFiles[Parse file mappings]
    LoadEffects --> ParseHooks[Parse hook commands]
    
    ParseFiles --> ValidateFiles[Validate file effects]
    ParseHooks --> ValidateHooks[Validate hook effects]
    
    ValidateFiles --> EffectRegistry[Effect Registry]
    ValidateHooks --> EffectRegistry
    Defaults --> EffectRegistry
    
    EffectRegistry --> Ready[Effects ready to apply]
```

## Effect Execution Strategy

```mermaid
graph TD
    Operation[Worktree Operation] --> Phase{Which phase?}
    
    Phase -->|Create| CreateEffects[Creation Effects]
    Phase -->|Remove| RemoveEffects[Removal Effects]
    Phase -->|List| NoEffects[No Effects]
    
    CreateEffects --> PreCreate[Pre-Create Effects]
    PreCreate --> CreateWorktree[Create Worktree]
    CreateWorktree --> PostCreate[Post-Create Effects]
    
    PostCreate --> FileEffects[File Mapping Effects]
    FileEffects --> SymlinkEffect{Can create symlink?}
    
    SymlinkEffect -->|Yes| CreateSymlink[Create Symlink]
    SymlinkEffect -->|No| FallbackCopy[Copy File Instead]
    
    CreateSymlink --> HookEffects[Hook Effects]
    FallbackCopy --> HookEffects
    
    HookEffects --> ReportEffects[Report All Effects]
```

## Effect Context Propagation

The `WorktreeContext` flows through all effects, providing necessary information:

```mermaid
graph TD
    Context[WorktreeContext Created] --> Fields{Context Fields}
    
    Fields --> Branch[branch_name]
    Fields --> Path[worktree_path]
    Fields --> Source[source_path]
    
    Branch --> HookVars["{branch}" in hooks]
    Path --> HookPath["{worktree_path}" in hooks]
    Path --> SymlinkTarget[Symlink targets]
    Source --> SymlinkSource[Symlink sources]
    Source --> HookSource["{source_path}" in hooks]
```

## Effect Error Handling Flow

```mermaid
graph TD
    Effect[Execute Effect] --> Try{Success?}
    Try -->|Yes| NextEffect[Next Effect]
    Try -->|No| Critical{Critical effect?}
    
    Critical -->|Yes| Abort[Abort Operation]
    Critical -->|No| Recoverable{Recoverable?}
    
    Recoverable -->|Yes| Fallback[Try Fallback]
    Fallback --> FallbackSuccess{Success?}
    FallbackSuccess -->|Yes| NextEffect
    FallbackSuccess -->|No| LogWarning[Log Warning]
    
    Recoverable -->|No| LogError[Log Error]
    LogWarning --> Continue[Continue with next effect]
    LogError --> Continue
    
    Continue --> NextEffect
    Abort --> Rollback[Rollback if possible]
```

## Platform-Specific Effect Resolution

```mermaid
graph TD
    FileMapping[File Mapping Effect] --> Platform{Platform?}
    
    Platform -->|Windows| WinCheck{Developer Mode?}
    Platform -->|Unix/macOS| UnixSymlink[Create Symlink]
    
    WinCheck -->|Yes| WinSymlink[Create Symlink]
    WinCheck -->|No| AdminCheck{Admin Rights?}
    
    AdminCheck -->|Yes| WinSymlink
    AdminCheck -->|No| CopyFallback[Copy File]
    
    UnixSymlink --> Verify[Verify Effect]
    WinSymlink --> Verify
    CopyFallback --> Verify
    
    Verify --> Report[Report Effect Result]
```

## Hook Effect Variable Substitution

Variables in hook commands are replaced with context values:

```mermaid
graph LR
    Template["echo 'Branch: {branch}'"] --> Parse[Parse Variables]
    Parse --> Context[Get from Context]
    Context --> Replace[Replace Variables]
    Replace --> Final["echo 'Branch: feature-branch'"]
    Final --> Execute[Execute Command]
```

## Effect Result Aggregation

```mermaid
graph TD
    Effects[Multiple Effects] --> Collect[Collect Results]
    
    Collect --> Success[Successful Effects]
    Collect --> Warnings[Warning Effects]
    Collect --> Failures[Failed Effects]
    
    Success --> Summary[Create Summary]
    Warnings --> Summary
    Failures --> Summary
    
    Summary --> Format{Output Format}
    Format -->|Table| TableOut[Table with status]
    Format -->|JSON| JSONOut[Structured JSON]
    Format -->|Simple| SimpleOut[Plain text]
```

## Lifecycle Effect Timing

The timing of effects is crucial for proper operation:

```mermaid
gantt
    title Effect Execution Timeline
    dateFormat X
    axisFormat %s
    
    section Pre-Create
    Validate Config     :0, 1
    Pre-Create Hooks    :1, 2
    
    section Primary Operation
    Create Worktree     :2, 4
    
    section Post-Create
    Create Symlinks     :4, 5
    Copy Files          :4, 5
    Post-Create Hooks   :5, 7
    
    section Finalize
    Report Results      :7, 8
```

## Data Structures Flow

Key data structures and their transformation through the system:

```rust
// 1. User Input
CreateArgs { branch: "feature", path: None }
    ↓
// 2. Configuration + Defaults
Config { files: [...], hooks: [...] }
    ↓
// 3. Worktree Context
WorktreeContext {
    branch_name: "feature",
    worktree_path: "/path/to/feature",
    source_path: "/path/to/repo"
}
    ↓
// 4. Effect Execution
Vec<EffectResult> [
    SymlinkResult { success: true, ... },
    HookResult { success: true, ... }
]
    ↓
// 5. User Output
"✓ Created worktree 'feature' with 2 effects applied"
```

Source: [src/git.rs#L65-660](https://github.com/your-org/twin/blob/main/src/git.rs#L65-660), [src/hooks.rs#L75-118](https://github.com/your-org/twin/blob/main/src/hooks.rs#L75-118)
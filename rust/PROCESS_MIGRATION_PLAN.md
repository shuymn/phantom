# Process Operations Migration Plan to CommandExecutor

## Overview
This document outlines the migration plan for all process operations in the Phantom Rust project to use the CommandExecutor infrastructure. The CommandExecutor provides a unified, testable interface for executing external commands with support for mocking.

## Current State Analysis

### CommandExecutor Infrastructure
- **Location**: `src/core/command_executor.rs`
- **Status**: Complete and functional
- **Features**:
  - Async trait-based design with `execute()` and `spawn()` methods
  - `CommandConfig` for synchronous execution with output capture
  - `SpawnConfig` for spawning processes (detached or interactive)
  - Support for working directory, environment variables, and timeouts
  - Real implementation in `src/core/executors/real_executor.rs`
  - Mock implementation in `src/core/executors/mock_executor.rs`

### Process Operations Requiring Migration

#### 1. **spawn.rs** - Core Process Spawning
- **Current Implementation**: Direct use of `tokio::process::Command`
- **Functions**:
  - `spawn_process(SpawnConfig)` - Main spawning function
  - `spawn_detached(SpawnConfig)` - Spawn without waiting
  - `execute_command()` - Execute and capture output
  - `setup_signal_handlers()` - Unix signal handling
- **Used By**: All other process modules (tmux, kitty, shell, etc.)
- **Migration Complexity**: High - Central to all process operations

#### 2. **tmux.rs** - Tmux Terminal Multiplexer Operations
- **Current Implementation**: Uses `spawn_process()` and `execute_command()` from spawn.rs
- **Functions**:
  - `execute_tmux_command()` - Execute commands in tmux
  - `create_tmux_session()` - Create new tmux sessions
  - `attach_tmux_session()` - Attach to existing sessions
  - `list_tmux_sessions()` - List active sessions
  - `tmux_session_exists()` - Check session existence
- **Migration Complexity**: Medium - Well-structured, clear command patterns

#### 3. **kitty.rs** - Kitty Terminal Operations
- **Current Implementation**: Uses `spawn_process()` from spawn.rs
- **Functions**:
  - `execute_kitty_command()` - Execute commands in kitty
  - `is_inside_kitty()` - Detect kitty environment
- **Migration Complexity**: Low - Simple command structure

#### 4. **fzf.rs** - Fuzzy Finder Integration
- **Current Implementation**: Direct use of `std::process::Command` (not tokio)
- **Functions**:
  - `select_with_fzf()` - Interactive selection with fzf
  - `is_fzf_available()` - Check fzf installation
- **Special Considerations**: 
  - Requires stdin/stdout piping for interactive operation
  - Uses synchronous `std::process` instead of async
- **Migration Complexity**: High - Interactive process with special I/O requirements

#### 5. **shell.rs** - Shell Detection and Execution
- **Current Implementation**: Uses `spawn_process()` from spawn.rs
- **Functions**:
  - `detect_shell()` - Detect user's shell
  - `shell_in_dir()` - Open shell in directory
  - Environment variable management for phantom sessions
- **Migration Complexity**: Low - Simple command execution

#### 6. **multiplexer.rs** - Unified Multiplexer Interface
- **Current Implementation**: Orchestrates tmux/kitty operations
- **Functions**:
  - `execute_in_multiplexer()` - Route to appropriate multiplexer
  - `detect_multiplexer()` - Detect available multiplexer
- **Migration Complexity**: Low - Mostly delegation to other modules

#### 7. **exec.rs** - High-level Command Execution
- **Current Implementation**: Uses `spawn_process()` from spawn.rs
- **Functions**:
  - `exec_in_dir()` - Execute command in directory
  - `exec_in_worktree()` - Execute in worktree with environment
  - `spawn_shell_in_dir()` - Spawn shell in directory
  - `spawn_shell_in_worktree()` - Spawn shell in worktree
  - `exec_commands_in_dir()` - Execute multiple commands
- **Migration Complexity**: Medium - Coordinates with worktree validation

#### 8. **prompt.rs** - User Interaction
- **Current Implementation**: Direct use of `std::io` for stdin/stdout
- **Functions**:
  - `confirm()` - Yes/no prompts
  - `prompt()` - Text input prompts
  - `select()` - Selection from list
- **Migration Complexity**: Not applicable - No external process execution

#### 9. **tty.rs** - Terminal Detection
- **Current Implementation**: Uses `std::io::IsTerminal` and `terminal_size` crate
- **Functions**:
  - TTY detection for stdin/stdout/stderr
  - Terminal size detection
  - Color support detection
- **Migration Complexity**: Not applicable - No external process execution

## Migration Strategy

### Phase 1: Create Process-specific CommandExecutor Adapter
Create a process-specific adapter similar to `GitExecutorAdapter` to minimize changes:

```rust
// src/process/process_executor_adapter.rs
pub struct ProcessExecutorAdapter {
    executor: Arc<dyn CommandExecutor>,
}

impl ProcessExecutorAdapter {
    pub async fn spawn_process(&self, config: SpawnConfig) -> Result<SpawnSuccess> {
        // Adapt existing SpawnConfig to CommandExecutor's SpawnConfig
    }
    
    pub async fn execute_command(&self, command: &str, args: Vec<String>, cwd: Option<&Path>) -> Result<String> {
        // Adapt to CommandExecutor's execute method
    }
}
```

### Phase 2: Migration Order (by dependency)
1. **spawn.rs** - Create adapter and update core functions
2. **shell.rs** - Simple execution patterns
3. **kitty.rs** - Simple command structure
4. **tmux.rs** - More complex but well-structured
5. **exec.rs** - Update to use migrated spawn functions
6. **multiplexer.rs** - Update after tmux/kitty migration
7. **fzf.rs** - Special handling for interactive I/O

### Phase 3: Special Considerations

#### FZF Migration Challenge
FZF requires special handling due to its interactive nature:
- Need to extend CommandExecutor to support stdin piping
- May need a specialized `InteractiveCommandConfig` variant
- Consider keeping fzf.rs with direct process handling if too complex

#### Signal Handling
The `setup_signal_handlers()` function in spawn.rs doesn't execute external commands, so it doesn't need migration.

#### Testing Strategy
- Create mock tests for each migrated module
- Use `MockCommandExecutor` to test command construction
- Verify command arguments, environment variables, and working directories
- Test error scenarios (command not found, non-zero exit codes)

## Implementation Plan

### Step 1: Create ProcessExecutorAdapter
```rust
// Minimal adapter to wrap CommandExecutor for process operations
// This allows gradual migration without breaking existing code
```

### Step 2: Update spawn.rs
```rust
// Update spawn_process to use ProcessExecutorAdapter
// Keep the same public API to avoid breaking changes
```

### Step 3: Migrate Simple Modules
- shell.rs - Direct command execution
- kitty.rs - Simple command patterns

### Step 4: Migrate Complex Modules
- tmux.rs - Multiple command variations
- exec.rs - Coordination with worktree operations

### Step 5: Handle Special Cases
- fzf.rs - May need CommandExecutor extension or keep as-is
- multiplexer.rs - Update after dependencies are migrated

### Step 6: Cleanup
- Remove old SpawnConfig from spawn.rs (conflicts with CommandExecutor's)
- Consolidate error handling
- Update all tests

## Benefits of Migration

1. **Testability**: Mock command execution for unit tests
2. **Consistency**: Unified interface for all process operations
3. **Maintainability**: Centralized command execution logic
4. **Debugging**: Better logging and error handling
5. **Future Features**: Easy to add command retry, better timeout handling, etc.

## Risks and Mitigation

1. **Risk**: Breaking existing functionality
   - **Mitigation**: Use adapter pattern for gradual migration
   
2. **Risk**: Interactive processes (fzf) may not fit the model
   - **Mitigation**: Extend CommandExecutor or keep specialized handling
   
3. **Risk**: Performance overhead from abstraction
   - **Mitigation**: Minimal - CommandExecutor is lightweight

## Conclusion

The migration to CommandExecutor will improve the codebase's testability and maintainability. The adapter pattern allows for gradual migration without breaking existing functionality. Priority should be given to migrating after git operations are complete, as indicated in the project context.
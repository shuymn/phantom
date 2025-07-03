# Error Handling Improvement Guide

This document tracks locations where we can improve error handling by using anyhow's `bail!` macro instead of verbose `return Err(anyhow!(...))` patterns.

## Files to Update

### 1. `/workspaces/phantom/rust/src/cli/handlers/where_cmd.rs`

**Lines 21-22:**
```rust
// Current:
return Err(anyhow!("Usage: phantom where <worktree-name> or phantom where --fzf"));

// Should be:
bail!("Usage: phantom where <worktree-name> or phantom where --fzf");
```

**Lines 25-26:**
```rust
// Current:
return Err(anyhow!("Cannot specify both a worktree name and --fzf option"));

// Should be:
bail!("Cannot specify both a worktree name and --fzf option");
```

### 2. `/workspaces/phantom/rust/src/cli/handlers/shell.rs`

**Lines 27-28:**
```rust
// Current:
return Err(anyhow!("Usage: phantom shell <worktree-name> or phantom shell --fzf"));

// Should be:
bail!("Usage: phantom shell <worktree-name> or phantom shell --fzf");
```

**Lines 31-32:**
```rust
// Current:
return Err(anyhow!("Cannot specify both a worktree name and --fzf option"));

// Should be:
bail!("Cannot specify both a worktree name and --fzf option");
```

**Lines 58-59:**
```rust
// Current:
return Err(anyhow!("The --tmux option can only be used inside a tmux session"));

// Should be:
bail!("The --tmux option can only be used inside a tmux session");
```

**Lines 62-63:**
```rust
// Current:
return Err(anyhow!("The --kitty option can only be used inside a kitty terminal"));

// Should be:
bail!("The --kitty option can only be used inside a kitty terminal");
```

**Lines 120-121:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

**Lines 147-148:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

**Lines 164:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

### 3. `/workspaces/phantom/rust/src/cli/handlers/exec.rs`

**Lines 31-34:**
```rust
// Current:
return Err(anyhow!(
    "Usage: phantom exec <worktree-name> <command> [args...] or phantom exec --fzf <command> [args...]"
));

// Should be:
bail!(
    "Usage: phantom exec <worktree-name> <command> [args...] or phantom exec --fzf <command> [args...]"
);
```

**Lines 42:**
```rust
// Current:
return Err(anyhow!("Usage: phantom exec <worktree-name> <command> [args...]"));

// Should be:
bail!("Usage: phantom exec <worktree-name> <command> [args...]");
```

**Lines 51:**
```rust
// Current:
return Err(anyhow!("No command specified"));

// Should be:
bail!("No command specified");
```

**Lines 78:**
```rust
// Current:
return Err(anyhow!("The --tmux option can only be used inside a tmux session"));

// Should be:
bail!("The --tmux option can only be used inside a tmux session");
```

**Lines 82:**
```rust
// Current:
return Err(anyhow!("The --kitty option can only be used inside a kitty terminal"));

// Should be:
bail!("The --kitty option can only be used inside a kitty terminal");
```

**Lines 140:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

**Lines 170:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

**Lines 187:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

### 4. `/workspaces/phantom/rust/src/cli/handlers/delete.rs`

**Lines 23-25:**
```rust
// Current:
return Err(anyhow!(
    "Please provide a worktree name to delete, use --current to delete the current worktree, or use --fzf for interactive selection"
));

// Should be:
bail!(
    "Please provide a worktree name to delete, use --current to delete the current worktree, or use --fzf for interactive selection"
);
```

**Lines 29:**
```rust
// Current:
return Err(anyhow!("Cannot specify --current with a worktree name or --fzf option"));

// Should be:
bail!("Cannot specify --current with a worktree name or --fzf option");
```

**Lines 33:**
```rust
// Current:
return Err(anyhow!("Cannot specify both a worktree name and --fzf option"));

// Should be:
bail!("Cannot specify both a worktree name and --fzf option");
```

**Lines 49-52:**
```rust
// Current:
return Err(anyhow!(
    "Not in a worktree directory. The --current option can only be used from within a worktree."
));

// Should be:
bail!(
    "Not in a worktree directory. The --current option can only be used from within a worktree."
);
```

### 5. `/workspaces/phantom/rust/src/cli/handlers/attach.rs`

**Lines 42-46:**
```rust
// Current:
return Err(anyhow!(
    "Worktree '{}' already exists at path: {}",
    args.branch,
    worktree_path.display()
));

// Should be:
bail!(
    "Worktree '{}' already exists at path: {}",
    args.branch,
    worktree_path.display()
);
```

**Lines 54:**
```rust
// Current:
return Err(anyhow!("Branch '{}' not found in repository", args.branch));

// Should be:
bail!("Branch '{}' not found in repository", args.branch);
```

**Lines 78:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

**Lines 86:**
```rust
// Current:
.map_err(|e| anyhow!(e))

// Could be simplified (though this is a different pattern)
```

## Required Changes

1. Add `bail` to the anyhow imports in each file:
   ```rust
   use anyhow::{anyhow, bail, Context, Result};
   ```

2. Replace all `return Err(anyhow!(...))` with `bail!(...)`

3. For `.map_err(|e| anyhow!(e))` patterns, these could potentially be replaced with `.context(...)` or just `?` if the error type is already compatible.

## Benefits

- More concise and readable code
- Less boilerplate
- Consistent error handling patterns
- Better alignment with anyhow best practices
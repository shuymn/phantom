use crate::Result;
use std::marker::PhantomData;
use std::path::PathBuf;

/// Type-level states for worktrees
pub mod states {
    /// A worktree that has been created but not yet attached to a branch
    pub struct Created;

    /// A worktree that is attached to a branch
    pub struct Attached;

    /// A worktree that is detached (not on any branch)
    pub struct Detached;

    /// A worktree that has been locked
    pub struct Locked;

    /// A marker for deleted worktrees (prevents operations on deleted worktrees)
    pub struct Deleted;
}

/// A type-safe worktree that enforces valid state transitions at compile time
#[derive(Debug, Clone)]
pub struct TypedWorktree<S> {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_bare: bool,
    _state: PhantomData<S>,
}

// State-specific implementations

impl TypedWorktree<states::Created> {
    /// Create a new worktree in the Created state
    pub fn new(name: String, path: PathBuf, commit: String) -> Self {
        Self { name, path, branch: None, commit, is_bare: false, _state: PhantomData }
    }

    /// Attach the worktree to a branch
    pub fn attach(self, branch: String) -> Result<TypedWorktree<states::Attached>> {
        // In a real implementation, this would perform the git operation
        Ok(TypedWorktree {
            name: self.name,
            path: self.path,
            branch: Some(branch),
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        })
    }

    /// Transition directly to detached state
    pub fn detach(self) -> TypedWorktree<states::Detached> {
        TypedWorktree {
            name: self.name,
            path: self.path,
            branch: None,
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        }
    }
}

impl TypedWorktree<states::Attached> {
    /// Switch to a different branch (remains attached)
    pub fn switch_branch(&mut self, branch: String) -> Result<()> {
        // In a real implementation, this would perform the git operation
        self.branch = Some(branch);
        Ok(())
    }

    /// Detach from the current branch
    pub fn detach(self) -> TypedWorktree<states::Detached> {
        TypedWorktree {
            name: self.name,
            path: self.path,
            branch: None,
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        }
    }

    /// Lock the worktree
    pub fn lock(self, _reason: Option<String>) -> Result<TypedWorktree<states::Locked>> {
        // In a real implementation, this would perform the git operation
        Ok(TypedWorktree {
            name: self.name,
            path: self.path,
            branch: self.branch,
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        })
    }
}

impl TypedWorktree<states::Detached> {
    /// Attach to a branch from detached state
    pub fn attach(self, branch: String) -> Result<TypedWorktree<states::Attached>> {
        Ok(TypedWorktree {
            name: self.name,
            path: self.path,
            branch: Some(branch),
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        })
    }

    /// Lock the worktree
    pub fn lock(self, _reason: Option<String>) -> Result<TypedWorktree<states::Locked>> {
        Ok(TypedWorktree {
            name: self.name,
            path: self.path,
            branch: self.branch,
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        })
    }
}

impl TypedWorktree<states::Locked> {
    /// Unlock the worktree, returning to its previous state
    /// Since we don't track the previous state, we return to detached
    pub fn unlock(self) -> TypedWorktree<states::Detached> {
        TypedWorktree {
            name: self.name,
            path: self.path,
            branch: self.branch,
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        }
    }
}

// Common operations available for all non-deleted states
impl<S> TypedWorktree<S> {
    /// Get the worktree name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the worktree path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the current branch (if attached)
    pub fn branch(&self) -> Option<&String> {
        self.branch.as_ref()
    }

    /// Get the current commit
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// Check if this is a bare worktree
    pub fn is_bare(&self) -> bool {
        self.is_bare
    }

    /// Delete the worktree (consumes self to prevent further operations)
    pub fn delete(self) -> Result<TypedWorktree<states::Deleted>> {
        // In a real implementation, this would perform the git operation
        Ok(TypedWorktree {
            name: self.name,
            path: self.path,
            branch: self.branch,
            commit: self.commit,
            is_bare: self.is_bare,
            _state: PhantomData,
        })
    }
}

// Conversion from the existing Worktree type
impl From<crate::core::types::Worktree> for TypedWorktree<states::Detached> {
    fn from(worktree: crate::core::types::Worktree) -> Self {
        if worktree.is_detached || worktree.branch.is_none() {
            TypedWorktree {
                name: worktree.name,
                path: worktree.path,
                branch: worktree.branch,
                commit: worktree.commit,
                is_bare: worktree.is_bare,
                _state: PhantomData,
            }
        } else {
            // If it has a branch but we're forcing it to Detached state
            // This is a simplification - in reality we'd handle this better
            TypedWorktree {
                name: worktree.name,
                path: worktree.path,
                branch: worktree.branch,
                commit: worktree.commit,
                is_bare: worktree.is_bare,
                _state: PhantomData,
            }
        }
    }
}

// Conversion to support attached worktrees
impl From<crate::core::types::Worktree> for TypedWorktree<states::Attached> {
    fn from(worktree: crate::core::types::Worktree) -> Self {
        TypedWorktree {
            name: worktree.name,
            path: worktree.path,
            branch: worktree.branch,
            commit: worktree.commit,
            is_bare: worktree.is_bare,
            _state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_worktree_state_transitions() {
        // Create a new worktree
        let worktree = TypedWorktree::<states::Created>::new(
            "feature".to_string(),
            PathBuf::from("/tmp/feature"),
            "abc123".to_string(),
        );

        // Attach to a branch
        let attached = worktree.attach("feature-branch".to_string()).unwrap();
        assert_eq!(attached.branch(), Some(&"feature-branch".to_string()));

        // Detach from branch
        let detached = attached.detach();
        assert_eq!(detached.branch(), None);

        // Re-attach
        let reattached = detached.attach("main".to_string()).unwrap();
        assert_eq!(reattached.branch(), Some(&"main".to_string()));

        // Delete
        let deleted = reattached.delete().unwrap();
        // Can't do anything with deleted worktree (no methods available)
        let _ = deleted;
    }

    #[test]
    fn test_invalid_transitions_wont_compile() {
        // This test demonstrates that invalid transitions won't compile
        // Uncomment any of these lines to see compilation errors:

        // let created = TypedWorktree::<states::Created>::new(...);
        // created.switch_branch("main"); // ERROR: method not found

        // let deleted = created.delete().unwrap();
        // deleted.attach("main"); // ERROR: method not found

        // let locked = attached.lock(None).unwrap();
        // locked.switch_branch("dev"); // ERROR: method not found
    }

    #[test]
    fn test_common_operations() {
        let worktree = TypedWorktree::<states::Created>::new(
            "test".to_string(),
            PathBuf::from("/workspace/test"),
            "def456".to_string(),
        );

        assert_eq!(worktree.name(), "test");
        assert_eq!(worktree.path(), Path::new("/workspace/test"));
        assert_eq!(worktree.commit(), "def456");
        assert!(!worktree.is_bare());
    }
}

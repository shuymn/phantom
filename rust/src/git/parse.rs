use crate::core::types::Worktree;
use crate::git::const_utils::{is_branch_ref, REFS_HEADS_PREFIX};
use std::path::PathBuf;

/// Parse git worktree list output
pub fn parse_worktree_list(output: &str) -> Vec<Worktree> {
    let mut worktrees = Vec::new();
    let mut current_worktree: Option<WorktreeBuilder> = None;

    for line in output.lines() {
        if line.is_empty() {
            if let Some(builder) = current_worktree.take() {
                if let Some(worktree) = builder.build() {
                    worktrees.push(worktree);
                }
            }
            continue;
        }

        // Handle lines with values (key value) and without values (just key)
        let (key, value) =
            if let Some((k, v)) = line.split_once(' ') { (k, Some(v)) } else { (line, None) };

        match key {
            "worktree" => {
                if let Some(builder) = current_worktree.take() {
                    if let Some(worktree) = builder.build() {
                        worktrees.push(worktree);
                    }
                }
                if let Some(v) = value {
                    current_worktree = Some(WorktreeBuilder::new(v));
                }
            }
            "HEAD" => {
                if let Some(ref mut builder) = current_worktree {
                    if let Some(v) = value {
                        builder.commit = Some(v.to_string());
                    }
                }
            }
            "branch" => {
                if let Some(ref mut builder) = current_worktree {
                    if let Some(v) = value {
                        // Parse branch format: refs/heads/branch-name
                        let branch_name = if is_branch_ref(v) {
                            v.strip_prefix(REFS_HEADS_PREFIX).unwrap().to_string()
                        } else {
                            v.to_string()
                        };
                        builder.branch = Some(branch_name);
                    }
                }
            }
            "bare" => {
                if let Some(ref mut builder) = current_worktree {
                    builder.is_bare = true;
                }
            }
            "detached" => {
                if let Some(ref mut builder) = current_worktree {
                    builder.is_detached = true;
                    builder.branch = None; // Detached HEAD has no branch
                }
            }
            "locked" => {
                if let Some(ref mut builder) = current_worktree {
                    builder.is_locked = true;
                }
            }
            "prunable" => {
                if let Some(ref mut builder) = current_worktree {
                    builder.is_prunable = true;
                }
            }
            _ => {} // Ignore other fields
        }
    }

    // Don't forget the last worktree
    if let Some(builder) = current_worktree {
        if let Some(worktree) = builder.build() {
            worktrees.push(worktree);
        }
    }

    worktrees
}

/// Parse git branch list output
pub fn parse_branch_list(output: &str) -> Vec<(String, bool)> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            if let Some(branch) = line.strip_prefix("* ") {
                Some((branch.to_string(), true)) // Current branch
            } else {
                Some((line.trim().to_string(), false))
            }
        })
        .collect()
}

/// Builder for Worktree
#[derive(Debug)]
struct WorktreeBuilder {
    path: PathBuf,
    commit: Option<String>,
    branch: Option<String>,
    is_bare: bool,
    is_detached: bool,
    is_locked: bool,
    is_prunable: bool,
}

impl WorktreeBuilder {
    fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path),
            commit: None,
            branch: None,
            is_bare: false,
            is_detached: false,
            is_locked: false,
            is_prunable: false,
        }
    }

    fn build(self) -> Option<Worktree> {
        let name = self.path.file_name()?.to_string_lossy().to_string();
        let commit = self.commit?;

        Some(Worktree {
            name,
            path: self.path,
            branch: self.branch,
            commit,
            is_bare: self.is_bare,
            is_detached: self.is_detached,
            is_locked: self.is_locked,
            is_prunable: self.is_prunable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list() {
        let output = r#"worktree /path/to/repo
HEAD abc123def456
branch refs/heads/main

worktree /path/to/repo/feature-branch
HEAD 789012345678
branch refs/heads/feature-branch
"#;

        let worktrees = parse_worktree_list(output);
        assert_eq!(worktrees.len(), 2);

        assert_eq!(worktrees[0].name, "repo");
        assert_eq!(worktrees[0].path, PathBuf::from("/path/to/repo"));
        assert_eq!(worktrees[0].branch, Some("main".to_string()));
        assert_eq!(worktrees[0].commit, "abc123def456");
        assert!(!worktrees[0].is_bare);
        assert!(!worktrees[0].is_detached);
        assert!(!worktrees[0].is_locked);

        assert_eq!(worktrees[1].name, "feature-branch");
        assert_eq!(worktrees[1].path, PathBuf::from("/path/to/repo/feature-branch"));
        assert_eq!(worktrees[1].branch, Some("feature-branch".to_string()));
    }

    #[test]
    fn test_parse_worktree_list_with_detached() {
        let output = r#"worktree /path/to/repo
HEAD abc123def456
detached
"#;

        let worktrees = parse_worktree_list(output);
        assert_eq!(worktrees.len(), 1);
        assert!(worktrees[0].is_detached);
        assert!(worktrees[0].branch.is_none());
        assert!(!worktrees[0].is_locked);
    }

    #[test]
    fn test_parse_branch_list() {
        let output = r#"  main
* feature-branch
  develop
  hotfix/issue-123"#;

        let branches = parse_branch_list(output);
        assert_eq!(branches.len(), 4);

        assert_eq!(branches[0], ("main".to_string(), false));
        assert_eq!(branches[1], ("feature-branch".to_string(), true));
        assert_eq!(branches[2], ("develop".to_string(), false));
        assert_eq!(branches[3], ("hotfix/issue-123".to_string(), false));
    }

    #[test]
    fn test_parse_branch_list_empty() {
        let output = "";
        let branches = parse_branch_list(output);
        assert!(branches.is_empty());
    }

    #[test]
    fn test_parse_worktree_list_with_all_flags() {
        let output = r#"worktree /path/to/repo
HEAD abc123def456
branch refs/heads/main
bare
locked
prunable

worktree /path/to/repo/test
HEAD def456abc123
detached
locked
"#;

        let worktrees = parse_worktree_list(output);
        assert_eq!(worktrees.len(), 2);

        // First worktree with bare, locked, and prunable
        assert_eq!(worktrees[0].name, "repo");
        assert!(worktrees[0].is_bare);
        assert!(worktrees[0].is_locked);
        assert!(worktrees[0].is_prunable);
        assert!(!worktrees[0].is_detached);

        // Second worktree with detached and locked
        assert_eq!(worktrees[1].name, "test");
        assert!(worktrees[1].is_detached);
        assert!(worktrees[1].is_locked);
        assert!(!worktrees[1].is_bare);
        assert!(!worktrees[1].is_prunable);
        assert!(worktrees[1].branch.is_none()); // detached has no branch
    }

    #[test]
    fn test_parse_worktree_list_invalid_entries() {
        // Test with worktree missing HEAD or commit
        let output = r#"worktree /path/to/repo
branch refs/heads/main

worktree /path/to/valid
HEAD abc123
"#;

        let worktrees = parse_worktree_list(output);
        assert_eq!(worktrees.len(), 1); // Only valid worktree is included
        assert_eq!(worktrees[0].name, "valid");
    }

    #[test]
    fn test_parse_worktree_list_with_unknown_fields() {
        let output = r#"worktree /path/to/repo
HEAD abc123def456
branch refs/heads/main
unknown_field some_value
another_unknown field
"#;

        let worktrees = parse_worktree_list(output);
        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].name, "repo");
        // Unknown fields should be ignored
    }

    #[test]
    fn test_parse_branch_list_with_whitespace() {
        let output = r#"  
  main
    
* feature-branch  
  develop
  
"#;

        let branches = parse_branch_list(output);
        assert_eq!(branches.len(), 3); // Empty lines are filtered out
        assert_eq!(branches[0], ("main".to_string(), false));
        assert_eq!(branches[1], ("feature-branch".to_string(), true));
        assert_eq!(branches[2], ("develop".to_string(), false));
    }

    #[test]
    fn test_worktree_builder_without_path_name() {
        // Test edge case where path has no file name (shouldn't happen in practice)
        let builder = WorktreeBuilder {
            path: PathBuf::from("/"),
            commit: Some("abc123".to_string()),
            branch: Some("main".to_string()),
            is_bare: false,
            is_detached: false,
            is_locked: false,
            is_prunable: false,
        };

        // This should return None because "/" has no file name
        assert!(builder.build().is_none());
    }

    #[test]
    fn test_parse_worktree_list_final_worktree() {
        // Test that the last worktree without trailing empty line is parsed
        let output = r#"worktree /path/to/repo
HEAD abc123def456
branch refs/heads/main

worktree /path/to/repo/last
HEAD def789ghi012
branch refs/heads/last-branch"#; // No trailing newline

        let worktrees = parse_worktree_list(output);
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[1].name, "last");
        assert_eq!(worktrees[1].branch, Some("last-branch".to_string()));
    }

    #[test]
    fn test_parse_branch_with_special_chars() {
        let output = r#"  feature/ABC-123
* bugfix/DEF-456  
  release/v1.2.3
  user@feature/test"#;

        let branches = parse_branch_list(output);
        assert_eq!(branches.len(), 4);
        assert_eq!(branches[0], ("feature/ABC-123".to_string(), false));
        assert_eq!(branches[1], ("bugfix/DEF-456".to_string(), true));
        assert_eq!(branches[2], ("release/v1.2.3".to_string(), false));
        assert_eq!(branches[3], ("user@feature/test".to_string(), false));
    }

    #[test]
    fn test_worktree_builder_debug() {
        let builder = WorktreeBuilder::new("/path/to/repo");
        let debug_str = format!("{builder:?}");
        assert!(debug_str.contains("WorktreeBuilder"));
        assert!(debug_str.contains("/path/to/repo"));
    }
}

use crate::core::types::Worktree;
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
        let (key, value) = if let Some((k, v)) = line.split_once(' ') {
            (k, Some(v))
        } else {
            (line, None)
        };
        
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
                            let branch_name = v.strip_prefix("refs/heads/")
                                .unwrap_or(v)
                                .to_string();
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
                Some((branch.to_string(), true))  // Current branch
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
}
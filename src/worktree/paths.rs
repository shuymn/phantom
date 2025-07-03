use crate::worktree::const_validate::DEFAULT_PHANTOM_DIR;
use std::path::{Path, PathBuf};

/// Get the phantom directory path within the git repository
pub fn get_phantom_directory(git_root: &Path) -> PathBuf {
    git_root.join(DEFAULT_PHANTOM_DIR)
}

/// Get the path for a specific worktree
pub fn get_worktree_path(git_root: &Path, name: &str) -> PathBuf {
    get_phantom_directory(git_root).join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_phantom_directory() {
        let git_root = Path::new("/home/user/project");
        let phantom_dir = get_phantom_directory(git_root);
        assert_eq!(phantom_dir, PathBuf::from("/home/user/project/.git/phantom/worktrees"));
    }

    #[test]
    fn test_get_worktree_path() {
        let git_root = Path::new("/home/user/project");
        let worktree_path = get_worktree_path(git_root, "feature-branch");
        assert_eq!(
            worktree_path,
            PathBuf::from("/home/user/project/.git/phantom/worktrees/feature-branch")
        );
    }

    #[test]
    fn test_get_worktree_path_with_slashes() {
        let git_root = Path::new("/home/user/project");
        let worktree_path = get_worktree_path(git_root, "feature/sub-feature");
        assert_eq!(
            worktree_path,
            PathBuf::from("/home/user/project/.git/phantom/worktrees/feature/sub-feature")
        );
    }
}

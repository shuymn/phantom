use crate::git::executor::GitExecutor;
use crate::git::libs::list_worktrees::list_worktrees;
use crate::Result;
use std::path::Path;
use tracing::debug;

/// Get the current worktree branch name (returns None if in main worktree)
pub async fn get_current_worktree(git_root: &Path) -> Result<Option<String>> {
    // Get the current working directory's git root
    let executor = GitExecutor::new();
    let current_path = executor.run(&["rev-parse", "--show-toplevel"]).await?;
    let current_path = current_path.trim();
    let current_path = Path::new(current_path);
    
    debug!("Current worktree path: {:?}", current_path);
    
    // Get all worktrees
    let worktrees = list_worktrees(git_root).await?;
    
    // Find the current worktree
    let current_worktree = worktrees.into_iter()
        .find(|wt| wt.path == current_path);
    
    match current_worktree {
        Some(wt) => {
            // Canonicalize paths for comparison
            let wt_canonical = wt.path.canonicalize().unwrap_or(wt.path.clone());
            let git_root_canonical = git_root.canonicalize().unwrap_or(git_root.to_path_buf());
            
            if wt_canonical != git_root_canonical {
                debug!("Current worktree branch: {:?}", wt.branch);
                Ok(wt.branch)
            } else {
                debug!("In main worktree");
                Ok(None)
            }
        }
        None => {
            debug!("Worktree not found");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;
    use crate::git::executor::GitExecutor;
    use std::env;

    #[tokio::test]
    async fn test_get_current_worktree_main() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Change to the main repo directory
        let _guard = TestWorkingDir::new(repo.path());
        
        let result = get_current_worktree(repo.path()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_current_worktree_in_worktree() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Create a worktree with unique name
        let executor = GitExecutor::with_cwd(repo.path());
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("test-worktree-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor.run(&["worktree", "add", "-b", "feature-branch", &worktree_path.to_string_lossy()])
            .await
            .unwrap();
        
        // Change to the worktree directory
        let _guard = TestWorkingDir::new(&worktree_path);
        
        let result = get_current_worktree(repo.path()).await.unwrap();
        assert_eq!(result, Some("feature-branch".to_string()));
    }

    #[tokio::test]
    async fn test_get_current_worktree_detached() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Get the current commit
        let executor = GitExecutor::with_cwd(repo.path());
        let commit = executor.run(&["rev-parse", "HEAD"]).await.unwrap();
        let commit = commit.trim();
        
        // Create a detached worktree with unique name
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("detached-worktree-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor.run(&["worktree", "add", "--detach", &worktree_path.to_string_lossy(), commit])
            .await
            .unwrap();
        
        // Change to the worktree directory
        let _guard = TestWorkingDir::new(&worktree_path);
        
        let result = get_current_worktree(repo.path()).await.unwrap();
        assert_eq!(result, None); // Detached worktree has no branch
    }

    /// Helper struct to temporarily change working directory
    struct TestWorkingDir {
        original: std::path::PathBuf,
    }
    
    impl TestWorkingDir {
        fn new(path: &Path) -> Self {
            let original = env::current_dir().unwrap();
            env::set_current_dir(path).unwrap();
            Self { original }
        }
    }
    
    impl Drop for TestWorkingDir {
        fn drop(&mut self) {
            env::set_current_dir(&self.original).unwrap();
        }
    }
}
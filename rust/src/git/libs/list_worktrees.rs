use crate::core::types::Worktree;
use crate::git::executor::GitExecutor;
use crate::git::parse::parse_worktree_list;
use crate::Result;
use std::path::Path;
use tracing::debug;

/// List all git worktrees in a repository
pub async fn list_worktrees(repo_path: &Path) -> Result<Vec<Worktree>> {
    let executor = GitExecutor::with_cwd(repo_path);
    
    debug!("Listing worktrees in {:?}", repo_path);
    let output = executor.run(&["worktree", "list", "--porcelain"]).await?;
    
    let worktrees = parse_worktree_list(&output);
    debug!("Found {} worktrees", worktrees.len());
    
    Ok(worktrees)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_list_worktrees_single() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        let worktrees = list_worktrees(repo.path()).await.unwrap();
        
        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].name, repo.path().file_name().unwrap().to_str().unwrap());
        assert!(!worktrees[0].is_bare);
        assert!(!worktrees[0].is_detached);
        assert!(!worktrees[0].is_locked);
        assert!(!worktrees[0].is_prunable);
    }

    #[tokio::test]
    async fn test_list_worktrees_multiple() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Add a worktree with unique name
        let executor = GitExecutor::with_cwd(repo.path());
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("test-worktree-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor.run(&["worktree", "add", "-b", "test-branch", &worktree_path.to_string_lossy()])
            .await
            .unwrap();
        
        let worktrees = list_worktrees(repo.path()).await.unwrap();
        
        assert_eq!(worktrees.len(), 2);
        
        // Find the main worktree - need to canonicalize paths for comparison
        let repo_canonical = repo.path().canonicalize().unwrap();
        let main_worktree = worktrees.iter()
            .find(|w| w.path.canonicalize().unwrap() == repo_canonical)
            .expect("Main worktree not found");
        assert_eq!(main_worktree.branch, Some("main".to_string()));
        
        // Find the new worktree - need to canonicalize paths for comparison
        let worktree_canonical = worktree_path.canonicalize().unwrap();
        let test_worktree = worktrees.iter()
            .find(|w| w.path.canonicalize().unwrap() == worktree_canonical)
            .expect("Test worktree not found");
        assert_eq!(test_worktree.branch, Some("test-branch".to_string()));
        assert_eq!(test_worktree.name, unique_name);
    }

    #[tokio::test]
    async fn test_list_worktrees_with_detached() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Get the current commit
        let executor = GitExecutor::with_cwd(repo.path());
        let commit = executor.run(&["rev-parse", "HEAD"]).await.unwrap();
        let commit = commit.trim();
        
        // Add a detached worktree with unique name
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("detached-worktree-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor.run(&["worktree", "add", "--detach", &worktree_path.to_string_lossy(), commit])
            .await
            .unwrap();
        
        let worktrees = list_worktrees(repo.path()).await.unwrap();
        
        assert_eq!(worktrees.len(), 2);
        
        // Find the detached worktree - need to canonicalize paths for comparison
        let worktree_canonical = worktree_path.canonicalize().unwrap();
        let detached_worktree = worktrees.iter()
            .find(|w| w.path.canonicalize().unwrap() == worktree_canonical)
            .expect("Detached worktree not found");
        
        assert!(detached_worktree.is_detached);
        assert!(detached_worktree.branch.is_none());
    }
}
use crate::git::executor::GitExecutor;
use crate::Result;
use std::path::Path;
use tracing::debug;

/// Get the current branch name
pub async fn get_current_branch(repo_path: &Path) -> Result<String> {
    let executor = GitExecutor::with_cwd(repo_path);
    
    debug!("Getting current branch in {:?}", repo_path);
    let output = executor.run(&["branch", "--show-current"]).await?;
    
    let branch = output.trim().to_string();
    debug!("Current branch: {}", branch);
    
    Ok(branch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;
    use crate::git::executor::GitExecutor;

    #[tokio::test]
    async fn test_get_current_branch_main() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "main");
    }

    #[tokio::test]
    async fn test_get_current_branch_after_checkout() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Create and checkout a new branch
        repo.create_branch("feature-branch").await.unwrap();
        let executor = GitExecutor::with_cwd(repo.path());
        executor.run(&["checkout", "feature-branch"]).await.unwrap();
        
        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "feature-branch");
    }

    #[tokio::test]
    async fn test_get_current_branch_detached_head() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Get the current commit
        let executor = GitExecutor::with_cwd(repo.path());
        let commit = executor.run(&["rev-parse", "HEAD"]).await.unwrap();
        let commit = commit.trim();
        
        // Checkout the commit directly (detached HEAD)
        executor.run(&["checkout", commit]).await.unwrap();
        
        // In detached HEAD state, --show-current returns empty
        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "");
    }

    #[tokio::test]
    async fn test_get_current_branch_with_dashes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Create and checkout a branch with dashes
        repo.create_branch("feature-with-dashes").await.unwrap();
        let executor = GitExecutor::with_cwd(repo.path());
        executor.run(&["checkout", "feature-with-dashes"]).await.unwrap();
        
        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "feature-with-dashes");
    }

    #[tokio::test]
    async fn test_get_current_branch_with_slashes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit")
            .await
            .unwrap();
        
        // Create and checkout a branch with slashes
        let executor = GitExecutor::with_cwd(repo.path());
        executor.run(&["checkout", "-b", "feature/new-feature"]).await.unwrap();
        
        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "feature/new-feature");
    }
}
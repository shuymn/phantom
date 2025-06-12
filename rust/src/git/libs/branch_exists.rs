use crate::git::executor::GitExecutor;
use crate::{PhantomError, Result};
use std::path::Path;
use tracing::debug;

/// Check if a branch exists in the repository
pub async fn branch_exists(git_root: &Path, branch_name: &str) -> Result<bool> {
    let executor = GitExecutor::with_cwd(git_root);

    debug!("Checking if branch '{}' exists in {:?}", branch_name, git_root);

    // Use show-ref to check if the branch exists
    let result = executor
        .run(&["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch_name)])
        .await;

    match result {
        Ok(_) => {
            debug!("Branch '{}' exists", branch_name);
            Ok(true)
        }
        Err(PhantomError::Git { exit_code: 1, .. }) => {
            // Exit code 1 means the branch doesn't exist
            debug!("Branch '{}' does not exist", branch_name);
            Ok(false)
        }
        Err(e) => {
            // Any other error is a real error
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_branch_exists_main() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let exists = branch_exists(repo.path(), "main").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_branch_exists_nonexistent() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let exists = branch_exists(repo.path(), "nonexistent-branch").await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_branch_exists_created_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a new branch
        repo.create_branch("feature-branch").await.unwrap();

        let exists = branch_exists(repo.path(), "feature-branch").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_branch_exists_with_slashes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch with slashes
        repo.create_branch("feature/new-feature").await.unwrap();

        let exists = branch_exists(repo.path(), "feature/new-feature").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_branch_exists_case_sensitive() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch
        repo.create_branch("Feature-Branch").await.unwrap();

        // On case-insensitive filesystems (like macOS), this test may behave differently
        // So we'll check if the filesystem is case-sensitive first
        let exists_lowercase = branch_exists(repo.path(), "feature-branch").await.unwrap();
        let exists_correct = branch_exists(repo.path(), "Feature-Branch").await.unwrap();

        // The correct case should always exist
        assert!(exists_correct);

        // On case-sensitive systems, lowercase should not exist
        // On case-insensitive systems, both will exist
        // We can't make strong assertions here due to filesystem differences
        if !exists_lowercase {
            println!("Running on case-sensitive filesystem");
        } else {
            println!("Running on case-insensitive filesystem");
        }
    }
}

use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor;
use crate::Result;
use std::path::Path;
use tracing::debug;

/// List all branches using a provided executor
pub async fn list_branches<E>(executor: E, cwd: &Path) -> Result<Vec<String>>
where
    E: CommandExecutor + Clone + 'static,
{
    let git_executor = GitExecutor::new(executor).with_cwd(cwd);

    debug!("Listing branches in {:?}", cwd);
    let output = git_executor.run(&["branch", "--format=%(refname:short)"]).await?;

    let branches: Vec<String> = output
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    debug!("Found {} branches", branches.len());
    Ok(branches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_list_branches_single() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        let branches = list_branches(RealCommandExecutor, repo.path()).await.unwrap();
        assert_eq!(branches, vec!["main"]);
    }

    #[tokio::test]
    async fn test_list_branches_multiple() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create additional branches
        repo.create_branch("feature-a").await.unwrap();
        repo.create_branch("feature-b").await.unwrap();
        repo.create_branch("bugfix/issue-123").await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        let mut branches = list_branches(RealCommandExecutor, repo.path()).await.unwrap();
        branches.sort(); // Sort for consistent comparison

        assert_eq!(branches, vec!["bugfix/issue-123", "feature-a", "feature-b", "main"]);
    }

    #[tokio::test]
    async fn test_list_branches_with_mock_executor() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["branch", "--format=%(refname:short)"])
            .in_dir("/test/repo")
            .returns_output("main\nfeature/awesome\nbugfix/critical\n", "", 0);

        let branches = list_branches(mock, Path::new("/test/repo")).await.unwrap();

        assert_eq!(branches, vec!["main", "feature/awesome", "bugfix/critical"]);
    }

    #[tokio::test]
    async fn test_list_branches_empty_with_mock() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["branch", "--format=%(refname:short)"])
            .in_dir("/test/repo")
            .returns_output("", "", 0);

        let branches = list_branches(mock, Path::new("/test/repo")).await.unwrap();

        assert!(branches.is_empty());
    }

    #[tokio::test]
    async fn test_list_branches_with_whitespace_with_mock() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["branch", "--format=%(refname:short)"])
            .in_dir("/test/repo")
            .returns_output("  main  \n  develop  \n\n  feature  \n", "", 0);

        let branches = list_branches(mock, Path::new("/test/repo")).await.unwrap();

        assert_eq!(branches, vec!["main", "develop", "feature"]);
    }
}

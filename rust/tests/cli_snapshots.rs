#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use insta::assert_snapshot;

    #[test]
    fn test_help_output() {
        let mut cmd = Command::cargo_bin("phantom").unwrap();
        let output = cmd.arg("--help").output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_snapshot!(stdout);
    }

    #[test]
    fn test_version_output() {
        let mut cmd = Command::cargo_bin("phantom").unwrap();
        let output = cmd.arg("--version").output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        // Version output changes, so we'll just check the format
        assert!(stdout.starts_with("phantom "));
    }

    #[test]
    fn test_list_empty_output() {
        let mut cmd = Command::cargo_bin("phantom").unwrap();
        let output = cmd.arg("list").output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_snapshot!(stdout);
    }

    // Example of testing error output
    #[test]
    fn test_invalid_command() {
        let mut cmd = Command::cargo_bin("phantom").unwrap();
        let output = cmd.arg("invalid-command").output().unwrap();

        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_snapshot!(stderr);
    }
}

// Example of a module that could have snapshot tests for formatting functions
#[cfg(test)]
mod format_tests {
    #[allow(unused_imports)]
    use insta::assert_snapshot;

    // This would be used when we have actual formatting functions
    #[test]
    #[ignore = "CLI not yet implemented"]
    fn test_worktree_list_format() {
        // Example of what we might test
        let _worktrees = [
            ("main", "/path/to/main", true),
            ("feature-1", "/path/to/feature-1", false),
            ("bugfix", "/path/to/bugfix", false),
        ];

        // let output = format_worktree_list(&worktrees);
        // assert_snapshot!(output);
    }

    #[test]
    #[ignore = "CLI not yet implemented"]
    fn test_error_format() {
        // Example of testing error formatting
        // let error = PhantomError::Worktree("Not found".to_string());
        // let output = format_error(&error);
        // assert_snapshot!(output);
    }
}

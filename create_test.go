package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func TestExecuteCreate(t *testing.T) {
	// Create a temporary directory for test
	tempDir := t.TempDir()

	// Initialize a git repository in temp directory
	cmd := exec.Command("git", "init")
	cmd.Dir = tempDir
	if err := cmd.Run(); err != nil {
		t.Fatalf("Failed to initialize git repository: %v", err)
	}

	// Make an initial commit (worktree requires at least one commit)
	testFile := filepath.Join(tempDir, "README.md")
	if err := os.WriteFile(testFile, []byte("# Test Repository"), 0644); err != nil {
		t.Fatalf("Failed to create test file: %v", err)
	}

	cmd = exec.Command("git", "add", ".")
	cmd.Dir = tempDir
	if err := cmd.Run(); err != nil {
		t.Fatalf("Failed to add files: %v", err)
	}

	cmd = exec.Command("git", "commit", "-m", "Initial commit")
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(),
		"GIT_AUTHOR_NAME=Test",
		"GIT_AUTHOR_EMAIL=test@example.com",
		"GIT_COMMITTER_NAME=Test",
		"GIT_COMMITTER_EMAIL=test@example.com",
	)
	if err := cmd.Run(); err != nil {
		t.Fatalf("Failed to commit: %v", err)
	}

	// Change to the temp directory
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer os.Chdir(originalDir)

	if err := os.Chdir(tempDir); err != nil {
		t.Fatalf("Failed to change directory: %v", err)
	}

	// Test creating a phantom
	testCases := []struct {
		name        string
		phantomName string
		expectError bool
	}{
		{
			name:        "Create valid phantom",
			phantomName: "test-feature",
			expectError: false,
		},
		{
			name:        "Create another phantom",
			phantomName: "another-feature",
			expectError: false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			err := executeCreate(tc.phantomName)
			if tc.expectError && err == nil {
				t.Errorf("Expected error but got none")
			}
			if !tc.expectError && err != nil {
				t.Errorf("Unexpected error: %v", err)
			}

			if !tc.expectError {
				// Verify the worktree was created
				phantomPath := filepath.Join(tempDir, ".git", "phantom", tc.phantomName)
				if _, err := os.Stat(phantomPath); os.IsNotExist(err) {
					t.Errorf("Phantom directory was not created at %s", phantomPath)
				}

				// Verify the branch was created
				cmd := exec.Command("git", "branch", "--list", tc.phantomName)
				output, err := cmd.Output()
				if err != nil {
					t.Errorf("Failed to list branches: %v", err)
				}
				if len(output) == 0 {
					t.Errorf("Branch %s was not created", tc.phantomName)
				}
			}
		})
	}
}

func TestExecuteCreateDuplicateName(t *testing.T) {
	// Create a temporary directory for test
	tempDir := t.TempDir()

	// Initialize a git repository in temp directory
	cmd := exec.Command("git", "init")
	cmd.Dir = tempDir
	if err := cmd.Run(); err != nil {
		t.Fatalf("Failed to initialize git repository: %v", err)
	}

	// Make an initial commit
	testFile := filepath.Join(tempDir, "README.md")
	if err := os.WriteFile(testFile, []byte("# Test Repository"), 0644); err != nil {
		t.Fatalf("Failed to create test file: %v", err)
	}

	cmd = exec.Command("git", "add", ".")
	cmd.Dir = tempDir
	if err := cmd.Run(); err != nil {
		t.Fatalf("Failed to add files: %v", err)
	}

	cmd = exec.Command("git", "commit", "-m", "Initial commit")
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(),
		"GIT_AUTHOR_NAME=Test",
		"GIT_AUTHOR_EMAIL=test@example.com",
		"GIT_COMMITTER_NAME=Test",
		"GIT_COMMITTER_EMAIL=test@example.com",
	)
	if err := cmd.Run(); err != nil {
		t.Fatalf("Failed to commit: %v", err)
	}

	// Change to the temp directory
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer os.Chdir(originalDir)

	if err := os.Chdir(tempDir); err != nil {
		t.Fatalf("Failed to change directory: %v", err)
	}

	// Create first phantom
	phantomName := "duplicate-test"
	if err := executeCreate(phantomName); err != nil {
		t.Fatalf("Failed to create first phantom: %v", err)
	}

	// Try to create phantom with same name (should fail)
	err = executeCreate(phantomName)
	if err == nil {
		t.Errorf("Expected error when creating duplicate phantom, but got none")
	}
}

func TestExecuteCreateOutsideGitRepo(t *testing.T) {
	// Create a temporary directory (not a git repo)
	tempDir := t.TempDir()

	// Change to the temp directory
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer os.Chdir(originalDir)

	if err := os.Chdir(tempDir); err != nil {
		t.Fatalf("Failed to change directory: %v", err)
	}

	// Try to create phantom outside git repo (should fail)
	err = executeCreate("test-phantom")
	if err == nil {
		t.Errorf("Expected error when creating phantom outside git repo, but got none")
	}
}
package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func executeCreate(name string) error {
	// Get repository root
	cmd := exec.Command("git", "rev-parse", "--show-toplevel")
	output, err := cmd.Output()
	if err != nil {
		return fmt.Errorf("failed to get repository root: %w", err)
	}
	repoRoot := strings.TrimSpace(string(output))

	// Create phantom directory path
	phantomPath := filepath.Join(repoRoot, ".git", "phantom", name)

	// Create the worktree
	cmd = exec.Command("git", "worktree", "add", "-b", name, phantomPath)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("failed to create worktree: %w", err)
	}

	fmt.Printf("Successfully created phantom '%s' at %s\n", name, phantomPath)
	return nil
}
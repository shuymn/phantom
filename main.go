package main

import (
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	subcommand := os.Args[1]

	switch subcommand {
	case "create":
		if len(os.Args) < 3 {
			fmt.Fprintf(os.Stderr, "Error: create command requires a name argument\n")
			fmt.Fprintf(os.Stderr, "Usage: git phantom create <name>\n")
			os.Exit(1)
		}
		if err := executeCreate(os.Args[2]); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
	case "list":
		fmt.Println("TODO: Implement list command")
	case "add":
		fmt.Println("TODO: Implement add command")
	case "switch":
		fmt.Println("TODO: Implement switch command")
	case "remove":
		fmt.Println("TODO: Implement remove command")
	case "prune":
		fmt.Println("TODO: Implement prune command")
	case "help", "-h", "--help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "Unknown subcommand: %s\n", subcommand)
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println("git-phantom - A convenient CLI tool for git worktree management")
	fmt.Println()
	fmt.Println("Usage:")
	fmt.Println("  git phantom <command> [arguments]")
	fmt.Println()
	fmt.Println("Available commands:")
	fmt.Println("  create <name>     Create a new phantom worktree with branch")
	fmt.Println("  list              List all worktrees")
	fmt.Println("  add <path>        Create a new worktree")
	fmt.Println("  switch <path>     Switch to a worktree (output cd command)")
	fmt.Println("  remove <path>     Remove a worktree")
	fmt.Println("  prune             Clean up non-existent worktrees")
	fmt.Println("  help              Show this help message")
}
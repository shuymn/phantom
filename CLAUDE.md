# Claude Context for phantom

## Project Overview
This is the phantom project repository.

## Development Guidelines
- All files, issues, and pull requests in this repository must be written in English
- Follow existing code conventions and patterns when making changes
- Test all changes before committing
- Always run pnpm ready before committing

## Project Structure
- `README.md` - Main project documentation
- `docs/` - Additional documentation files
  - `first-design.md` - Initial design concept
  - `second-design.md` - Latest design concept
- `src/` - Source code
  - `bin/` - Executable entry points
    - `phantom.ts` - Main CLI entry point
    - `garden.ts` - Alias command for garden management
  - `commands/` - Top-level commands
    - `exec.ts` - Execute commands in a garden
    - `shell.ts` - Open interactive shell in a garden
  - `gardens/` - Garden (worktree) management
    - `commands/` - Garden-specific commands
      - `create.ts` - Create new gardens
      - `delete.ts` - Delete gardens
      - `list.ts` - List all gardens
      - `where.ts` - Get garden path
  - `git/` - Git utility functions
    - `libs/` - Git helper libraries

## Important Notes
- Use English for all communications and documentation
- Maintain consistency with existing code style

## Early Design
- Initial design concept is available in docs/first-design.md
- Latest design concept is available in docs/second-design.md

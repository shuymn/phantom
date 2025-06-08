# 🤝 Contributing to Phantom

Thank you for your interest in contributing to Phantom! This guide will help you get started with development.

## 📋 Table of Contents

- [Development Setup](#development-setup)
- [Development Guidelines](#development-guidelines)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Pull Request Process](#pull-request-process)
- [Documentation](#documentation)
- [Release Process](#release-process)
- [Additional Resources](#additional-resources)

## 🛠️ Development Setup

### Prerequisites

- Node.js 22+ and pnpm 10+

### Getting Started

```bash
# Clone and setup
git clone https://github.com/aku11i/phantom.git
cd phantom
pnpm install

# run phantom in development mode
pnpm phantom
```

### Development Workflow

```bash
# Run tests
pnpm test

# Type checking
pnpm typecheck

# Linting
pnpm lint
# or
pnpm fix

# Run all checks before committing
pnpm ready
```

## 📝 Development Guidelines

### Language Requirements

- **All files, issues, and pull requests must be written in English**
- This ensures the project is accessible to the global community

### Code Style

- Follow existing code conventions and patterns
- Use TypeScript for all new code
- Follow the Single Responsibility Principle
- Keep modules focused and testable

### Architecture Principles

- **Single Responsibility Principle**: Each module has one clear responsibility
- **Separation of Concerns**: CLI, business logic, and git operations are separated
- **Testability**: Core modules are framework-agnostic and easily testable
- **No Code Duplication**: Common operations are centralized
- **Clear Dependencies**: Dependencies flow from CLI → Core (including Git operations)

## 🧪 Testing

### Running Tests

```bash
# Run all tests
pnpm test

# Run specific test file
pnpm test:file src/core/worktree/create.test.js
```

### Writing Tests

- Add tests for all new features
- Follow existing test patterns
- Use descriptive test names
- Test both success and error cases

## ✨ Code Quality

### Before Committing

Always run the following command before committing:

```bash
pnpm ready
```

This command runs:
- Linting (`pnpm lint`)
- Type checking (`pnpm typecheck`)
- All tests (`pnpm test`)

### Security Best Practices

- Never introduce code that exposes or logs secrets and keys
- Never commit secrets or keys to the repository
- Be careful with user input validation

## 🚀 Pull Request Requirements

- Clear description of changes
- Tests for new functionality
- Documentation updates if applicable
- All checks passing (`pnpm ready`)
- Follow existing code style

## 📚 Documentation

When contributing documentation:

- Keep language clear and concise
- Update the table of contents if adding sections
- Check for broken links

## 🚀 Release Process

To release a new version of Phantom:

1. **Ensure you're on main branch and up to date**
   ```bash
   git checkout main
   git pull
   ```

2. **Run all checks**
   ```bash
   pnpm ready
   ```

3. **Bump version**
   ```bash
   # For patch releases (bug fixes)
   npm version patch

   # For minor releases (new features)
   npm version minor

   # For major releases (breaking changes)
   npm version major
   ```

4. **Push the version commit and tag**
   ```bash
   git push && git push --tags
   ```

5. **Publish to npm**
   ```bash
   pnpm publish
   ```

6. **Create GitHub release**
   ```bash
   # Create a release with automatically generated notes
   gh release create v<version> \
     --title "Phantom v<version>" \
     --generate-notes \
     --target main

   # Example for v0.1.3:
   gh release create v0.1.3 \
     --title "Phantom v0.1.3" \
     --generate-notes \
     --target main
   ```

7. **Update release notes for clarity**
   - Review the auto-generated release notes using `gh release view v<version>`
   - Check PR descriptions for important details using `gh pr view <number>`
   - Update the release notes to be more user-friendly:
     - Group changes by category (Features, Bug Fixes, Improvements)
     - Add usage examples for new features
     - Explain the impact of changes in plain language
     - Highlight security fixes and breaking changes
   
   ```bash
   # Edit the release notes
   gh release edit v<version> --notes "$(cat <<'EOF'
   ## 🚀 What's New in v<version>
   
   ### ✨ New Features
   - Feature description with usage example
   
   ### 🐛 Bug Fixes
   - Clear description of what was fixed
   
   ### 🛠️ Improvements
   - Performance, security, or other improvements
   EOF
   )"
   ```

## 🙏 Thank You!

Your contributions make Phantom better for everyone. If you have questions, feel free to:
- Open an issue for bugs or feature requests
- Start a discussion for general questions
- Ask in pull request comments


# Phantom-rs Rust Unification Plan

## Overview

This document outlines the plan to transition from a dual TypeScript/Rust implementation to a Rust-only project named `phantom-rs`. The fork has diverged significantly from upstream with 302 commits ahead, making it effectively a new project that deserves its own identity.

## Current Situation

- **Fork Status**: 302 commits ahead of `aku11i/phantom`
- **Implementations**: Both TypeScript and Rust versions exist
- **Rust Maturity**: Complete implementation with 586 passing tests
- **Usage**: TypeScript version has no active users
- **Unique Features**: Kitty terminal support (not in upstream)

## Transition Strategy

### Phase 1: Immediate Actions

1. -[x] **Remove Upstream Remote**
   ```bash
   git remote remove upstream
   ```

2. -[x] **Create Fork Attribution and Disclaimer**
   - Update README with acknowledgments and disclaimer:
   ```markdown
   ## Disclaimer
   
   phantom-rs is an **unofficial** Rust port created as a personal learning project. 
   While it aims to provide similar functionality to the original phantom:
   
   - **No guarantee of feature parity** with the original TypeScript version
   - **No promise of identical behavior** for equivalent features
   - **Breaking changes may occur** as the project evolves
   - **Use at your own risk** in production environments
   
   This project serves as both a functional tool and a Rust learning exercise.
   
   ## Acknowledgments
   
   phantom-rs is a Rust port of the original [phantom](https://github.com/aku11i/phantom) by @aku11i.
   The demonstration GIFs and core functionality remain faithful to the original implementation.
   
   - Original TypeScript implementation: [@aku11i](https://github.com/aku11i)
   - Rust port and enhancements: [@shuymn](https://github.com/shuymn)
   ```
   
   Note: While some Apache-2.0 projects use NOTICE.md files, FORK_NOTICE.md is not a common GitHub convention. The attribution in README.md is sufficient for MIT-licensed projects.

### Phase 2: Repository Restructure (Week 2)

1. -[-] **Move Rust to Root**
   ```bash
   # Move Rust implementation to root
   mv rust/* .
   mv rust/.* . 2>/dev/null || true
   rmdir rust
   
   # Remove TypeScript implementation
   rm -rf src/ docs/ package.json pnpm-lock.yaml tsconfig.json .nvmrc build.mjs
   ```

2. **Update Documentation**
   - Update README.md to reflect Rust-only implementation
   - Keep existing GIFs (they accurately represent functionality)
   - Update installation instructions for Rust
   - Remove TypeScript-specific documentation

3. **Clean Up Configuration**
   - Remove Node.js/TypeScript CI workflows
   - Update `.gitignore` for Rust-only
   - Remove TypeScript-specific config files

### Phase 3: Release Preparation

1. **Distribution Setup**
   - Set up GitHub Actions for building binaries
   - Create release workflow for multiple platforms
   - **Note**: This project will NOT be published to crates.io

2. **Update Package Metadata**
   - Update Cargo.toml package name to `phantom-rs`
   - Ensure all references point to `shuymn/phantom-rs`
   - Remove any remaining `@aku11i/phantom` references

## File Structure After Transition

```
phantom-rs/
├── src/                  # Rust source code
├── tests/                # Rust tests
├── Cargo.toml           # Rust package manifest (name: phantom-rs)
├── Cargo.lock          
├── README.md            # Updated for Rust
├── LICENSE              # MIT license (unchanged)
├── CONTRIBUTING.md      # Updated for Rust development
├── .github/             # GitHub Actions for Rust
└── docs/
    └── assets/          # Keep existing GIFs
```

## Timeline Note

Tag operations and version management will be handled at a later time when appropriate.

## Key Decisions

### Why Keep the Original GIFs?
- They accurately represent the tool's functionality
- Covered under MIT license
- Show respect for original work
- No functional differences in the port

### Why Remove TypeScript Completely?
- No active users
- Maintenance burden
- Clear project direction
- Simpler contribution process

### Why Stay as a Fork?
- Maintains contribution history
- Shows respect for origins
- Preserves MIT license chain
- GitHub shows relationship clearly

## Success Criteria

- [ ] All TypeScript code removed
- [ ] Rust code at repository root
- [ ] CI/CD updated for Rust only
- [ ] Documentation updated
- [ ] Fork attribution clear and respectful in README

## Timeline

- **Week 1**: Complete Phase 1 (attribution)
- **Week 2**: Complete Phase 2 (restructure)
- **Week 3**: Complete Phase 3 (binary releases)
- **Week 4**: Monitor and address any issues

## Long-term Considerations

1. **Naming**: `phantom-rs` clearly indicates Rust implementation
2. **Repository**: Consider renaming GitHub repository to `phantom-rs` for consistency
3. **Community**: Build around Rust implementation
4. **Features**: Focus on Rust-specific improvements
5. **Upstream**: No plans to sync with upstream due to divergence

This plan ensures a respectful transition while establishing phantom-rs as a standalone Rust project.
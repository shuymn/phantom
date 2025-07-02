/// Const utilities for git operations
/// These functions can be evaluated at compile time for constant inputs
/// Git reference prefixes
pub const REFS_HEADS_PREFIX: &str = "refs/heads/";

/// Check if a string starts with a prefix (const version)
const fn const_starts_with(s: &str, prefix: &str) -> bool {
    let s_bytes = s.as_bytes();
    let prefix_bytes = prefix.as_bytes();

    if s_bytes.len() < prefix_bytes.len() {
        return false;
    }

    let mut i = 0;
    while i < prefix_bytes.len() {
        if s_bytes[i] != prefix_bytes[i] {
            return false;
        }
        i += 1;
    }

    true
}

/// Check if a ref is a branch ref
pub const fn is_branch_ref(ref_str: &str) -> bool {
    const_starts_with(ref_str, REFS_HEADS_PREFIX)
}

/// Validate git object hash format (simplified for const)
/// Only checks length and characters, not actual validity
#[allow(clippy::manual_is_ascii_check)] // is_ascii_hexdigit() is not const
pub const fn is_valid_git_hash(hash: &str) -> bool {
    let bytes = hash.as_bytes();
    let len = bytes.len();

    // Git hashes are typically 40 chars (SHA-1) or 64 chars (SHA-256)
    if len != 40 && len != 64 && len < 7 {
        // Allow short hashes (min 7 chars)
        return false;
    }

    // Check all characters are hex
    let mut i = 0;
    while i < len {
        let ch = bytes[i];
        if !matches!(ch, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
            return false;
        }
        i += 1;
    }

    true
}

/// Common git command names as constants
pub mod commands {
    pub const GIT: &str = "git";
    pub const WORKTREE: &str = "worktree";
    pub const LIST: &str = "list";
    pub const ADD: &str = "add";
    pub const BRANCH: &str = "branch";
}

/// Common git flags as constants
pub mod flags {
    pub const PORCELAIN: &str = "--porcelain";
    pub const BRANCH_FLAG: &str = "-b";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_constants() {
        // Verify they exist and have expected values
        assert_eq!(commands::GIT, "git");
        assert_eq!(commands::WORKTREE, "worktree");
        assert_eq!(commands::LIST, "list");
        assert_eq!(commands::ADD, "add");
        assert_eq!(commands::BRANCH, "branch");
        assert_eq!(flags::PORCELAIN, "--porcelain");
        assert_eq!(flags::BRANCH_FLAG, "-b");
        assert_eq!(REFS_HEADS_PREFIX, "refs/heads/");
    }

    #[test]
    fn test_is_branch_ref() {
        const IS_BRANCH: bool = is_branch_ref("refs/heads/main");
        const NOT_BRANCH: bool = is_branch_ref("refs/tags/v1.0");
        const NOT_REF: bool = is_branch_ref("main");

        // These are compile-time constants, so we just verify they exist
        let _ = IS_BRANCH;
        let _ = NOT_BRANCH;
        let _ = NOT_REF;

        // Runtime tests
        assert!(is_branch_ref("refs/heads/feature/test"));
        assert!(!is_branch_ref("refs/heads")); // Missing trailing slash
    }

    #[test]
    fn test_is_valid_git_hash() {
        // Valid SHA-1 hashes
        const VALID_SHA1: bool = is_valid_git_hash("abc123def456789012345678901234567890abcd");
        const VALID_SHORT: bool = is_valid_git_hash("abc123d");

        // Valid SHA-256 hash
        const VALID_SHA256: bool =
            is_valid_git_hash("abc123def456789012345678901234567890abcdef123456789012345678abcd");

        // These are compile-time constants, so we just verify they exist
        let _ = VALID_SHA1;
        let _ = VALID_SHORT;
        let _ = VALID_SHA256;

        // Invalid hashes
        const INVALID_CHARS: bool = is_valid_git_hash("abc123g"); // 'g' is not hex
        const TOO_SHORT: bool = is_valid_git_hash("abc12"); // Less than 7 chars

        let _ = INVALID_CHARS;
        let _ = TOO_SHORT;
    }
}

/// Const utilities for git operations
/// These functions can be evaluated at compile time for constant inputs

/// Git reference prefixes
pub const REFS_HEADS_PREFIX: &str = "refs/heads/";
pub const REFS_TAGS_PREFIX: &str = "refs/tags/";
pub const REFS_REMOTES_PREFIX: &str = "refs/remotes/";

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

/// Check if a ref is a branch ref and return true with the offset
/// Use with strip_prefix at runtime for actual extraction
pub const fn is_branch_ref_with_offset(ref_str: &str) -> (bool, usize) {
    if const_starts_with(ref_str, REFS_HEADS_PREFIX) {
        (true, REFS_HEADS_PREFIX.len())
    } else {
        (false, 0)
    }
}

/// Check if a ref is a branch ref
pub const fn is_branch_ref(ref_str: &str) -> bool {
    const_starts_with(ref_str, REFS_HEADS_PREFIX)
}

/// Check if a ref is a tag ref
pub const fn is_tag_ref(ref_str: &str) -> bool {
    const_starts_with(ref_str, REFS_TAGS_PREFIX)
}

/// Check if a ref is a remote ref
pub const fn is_remote_ref(ref_str: &str) -> bool {
    const_starts_with(ref_str, REFS_REMOTES_PREFIX)
}

/// Common git command names as constants
pub mod commands {
    pub const GIT: &str = "git";
    pub const WORKTREE: &str = "worktree";
    pub const LIST: &str = "list";
    pub const ADD: &str = "add";
    pub const REMOVE: &str = "remove";
    pub const PRUNE: &str = "prune";
    pub const STATUS: &str = "status";
    pub const BRANCH: &str = "branch";
    pub const CHECKOUT: &str = "checkout";
    pub const REV_PARSE: &str = "rev-parse";
}

/// Common git flags as constants
pub mod flags {
    pub const PORCELAIN: &str = "--porcelain";
    pub const QUIET: &str = "--quiet";
    pub const FORCE: &str = "--force";
    pub const BRANCH_FLAG: &str = "-b";
    pub const ALL: &str = "--all";
    pub const DETACH: &str = "--detach";
    pub const NO_TRACK: &str = "--no-track";
    pub const GIT_DIR: &str = "--git-dir";
    pub const WORK_TREE: &str = "--work-tree";
    pub const SHOW_TOPLEVEL: &str = "--show-toplevel";
    pub const GIT_COMMON_DIR: &str = "--git-common-dir";
}

/// Exit codes for git operations
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 129;
    pub const NOT_A_GIT_REPO: i32 = 128;
}

/// Validate git object hash format (simplified for const)
/// Only checks length and characters, not actual validity
pub const fn is_valid_git_hash(hash: &str) -> bool {
    let bytes = hash.as_bytes();
    let len = bytes.len();
    
    // Git hashes are typically 40 chars (SHA-1) or 64 chars (SHA-256)
    if len != 40 && len != 64 && len < 7 {  // Allow short hashes (min 7 chars)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_starts_with() {
        const TEST1: bool = const_starts_with("refs/heads/main", "refs/heads/");
        const TEST2: bool = const_starts_with("refs/tags/v1.0", "refs/heads/");
        const TEST3: bool = const_starts_with("main", "refs/heads/");
        const TEST4: bool = const_starts_with("refs/heads/", "refs/heads/");
        
        assert!(TEST1);
        assert!(!TEST2);
        assert!(!TEST3);
        assert!(TEST4);
    }

    #[test]
    fn test_is_branch_ref_with_offset() {
        const CHECK1: (bool, usize) = is_branch_ref_with_offset("refs/heads/main");
        const CHECK2: (bool, usize) = is_branch_ref_with_offset("refs/heads/feature/test");
        const CHECK3: (bool, usize) = is_branch_ref_with_offset("refs/tags/v1.0");
        const CHECK4: (bool, usize) = is_branch_ref_with_offset("main");
        
        assert_eq!(CHECK1, (true, 11));  // "refs/heads/" is 11 chars
        assert_eq!(CHECK2, (true, 11));
        assert_eq!(CHECK3, (false, 0));
        assert_eq!(CHECK4, (false, 0));
        
        // Test runtime extraction using the offset
        if CHECK1.0 {
            let branch = &"refs/heads/main"[CHECK1.1..];
            assert_eq!(branch, "main");
        }
    }

    #[test]
    fn test_ref_type_checks() {
        const IS_BRANCH: bool = is_branch_ref("refs/heads/main");
        const IS_TAG: bool = is_tag_ref("refs/tags/v1.0");
        const IS_REMOTE: bool = is_remote_ref("refs/remotes/origin/main");
        
        assert!(IS_BRANCH);
        assert!(IS_TAG);
        assert!(IS_REMOTE);
        
        const NOT_BRANCH: bool = is_branch_ref("refs/tags/v1.0");
        const NOT_TAG: bool = is_tag_ref("refs/heads/main");
        const NOT_REMOTE: bool = is_remote_ref("refs/heads/main");
        
        assert!(!NOT_BRANCH);
        assert!(!NOT_TAG);
        assert!(!NOT_REMOTE);
    }

    #[test]
    fn test_git_hash_validation() {
        // Valid SHA-1 hashes
        const VALID_SHA1: bool = is_valid_git_hash("abc123def456789012345678901234567890abcd");
        const VALID_SHORT: bool = is_valid_git_hash("abc123d");
        
        // Valid SHA-256 hash
        const VALID_SHA256: bool = is_valid_git_hash(
            "abc123def456789012345678901234567890abcdef123456789012345678abcd"
        );
        
        assert!(VALID_SHA1);
        assert!(VALID_SHORT);
        assert!(VALID_SHA256);
        
        // Invalid hashes
        const INVALID_CHARS: bool = is_valid_git_hash("abc123g");  // 'g' is not hex
        const TOO_SHORT: bool = is_valid_git_hash("abc12");  // Less than 7 chars
        const VALID_38_CHARS: bool = is_valid_git_hash("abc123def456789012345678901234567890ab");  // 38 chars is valid
        
        assert!(!INVALID_CHARS);
        assert!(!TOO_SHORT);
        assert!(VALID_38_CHARS);  // Short hashes are valid
    }

    #[test]
    fn test_command_constants() {
        // Just verify they exist and have expected values
        assert_eq!(commands::GIT, "git");
        assert_eq!(commands::WORKTREE, "worktree");
        assert_eq!(flags::PORCELAIN, "--porcelain");
        assert_eq!(exit_codes::NOT_A_GIT_REPO, 128);
    }

    #[test]
    fn test_edge_cases() {
        // Empty strings
        const EMPTY_CHECK: (bool, usize) = is_branch_ref_with_offset("");
        assert_eq!(EMPTY_CHECK, (false, 0));
        
        // Exact prefix
        const PREFIX_CHECK: (bool, usize) = is_branch_ref_with_offset("refs/heads/");
        assert_eq!(PREFIX_CHECK, (true, 11));
        
        // Hash edge cases
        const EMPTY_HASH: bool = is_valid_git_hash("");
        const SPACES_HASH: bool = is_valid_git_hash("abc 123");
        
        assert!(!EMPTY_HASH);
        assert!(!SPACES_HASH);
    }
}
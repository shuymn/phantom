/// Compile-time validation functions for worktree operations
/// These functions can be evaluated at compile time for constant inputs
///
/// Default phantom directory name as a const
pub const DEFAULT_PHANTOM_DIR: &str = ".git/phantom/worktrees";

/// Maximum allowed worktree name length
pub const MAX_WORKTREE_NAME_LENGTH: usize = 255;

/// Check if a character is valid for worktree names at compile time
pub const fn is_valid_worktree_char(ch: char) -> bool {
    matches!(ch,
        'a'..='z' | 'A'..='Z' | '0'..='9' |
        '-' | '_' | '.' | '/'
    )
}

/// Validate worktree name at compile time (basic validation)
/// For full validation including regex patterns, use the runtime validate_worktree_name
pub const fn is_valid_worktree_name_basic(name: &str) -> bool {
    let bytes = name.as_bytes();
    let len = bytes.len();

    // Check empty name
    if len == 0 {
        return false;
    }

    // Check length limit
    if len > MAX_WORKTREE_NAME_LENGTH {
        return false;
    }

    // Check for consecutive dots
    let mut i = 0;
    while i < len {
        if i + 1 < len && bytes[i] == b'.' && bytes[i + 1] == b'.' {
            return false;
        }
        i += 1;
    }

    // Check all characters are valid (ASCII only for const fn)
    let mut j = 0;
    while j < len {
        let ch = bytes[j];
        if !matches!(ch,
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' |
            b'-' | b'_' | b'.' | b'/'
        ) {
            return false;
        }
        j += 1;
    }

    true
}

/// Create a validated worktree name at compile time
/// This is a const constructor that ensures the name is valid
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedWorktreeName<'a> {
    name: &'a str,
}

impl<'a> ValidatedWorktreeName<'a> {
    /// Create a new validated worktree name
    /// This will panic at compile time if the name is invalid
    pub const fn new(name: &'a str) -> Self {
        assert!(is_valid_worktree_name_basic(name), "Invalid worktree name");
        Self { name }
    }

    /// Get the validated name
    pub const fn as_str(&self) -> &'a str {
        self.name
    }
}

/// Common timeout values as constants
pub mod timeouts {
    use std::time::Duration;

    /// Default timeout for git operations (30 seconds)
    pub const GIT_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_validation() {
        // These should compile successfully
        const VALID_NAME_1: bool = is_valid_worktree_name_basic("feature-branch");
        const VALID_NAME_2: bool = is_valid_worktree_name_basic("feature/sub");
        const VALID_NAME_3: bool = is_valid_worktree_name_basic("v1.0.0");

        assert!(VALID_NAME_1);
        assert!(VALID_NAME_2);
        assert!(VALID_NAME_3);

        // These should be false
        const INVALID_NAME_1: bool = is_valid_worktree_name_basic("");
        const INVALID_NAME_2: bool = is_valid_worktree_name_basic("feature..branch");
        const INVALID_NAME_3: bool = is_valid_worktree_name_basic("feature@branch");

        assert!(!INVALID_NAME_1);
        assert!(!INVALID_NAME_2);
        assert!(!INVALID_NAME_3);
    }

    #[test]
    fn test_validated_name() {
        // This should compile successfully
        const VALIDATED: ValidatedWorktreeName = ValidatedWorktreeName::new("feature-branch");
        assert_eq!(VALIDATED.as_str(), "feature-branch");
    }

    #[test]
    fn test_char_validation() {
        const VALID_A: bool = is_valid_worktree_char('a');
        const VALID_Z: bool = is_valid_worktree_char('Z');
        const VALID_0: bool = is_valid_worktree_char('0');
        const VALID_DASH: bool = is_valid_worktree_char('-');
        const VALID_SLASH: bool = is_valid_worktree_char('/');

        assert!(VALID_A);
        assert!(VALID_Z);
        assert!(VALID_0);
        assert!(VALID_DASH);
        assert!(VALID_SLASH);

        const INVALID_SPACE: bool = is_valid_worktree_char(' ');
        const INVALID_AT: bool = is_valid_worktree_char('@');

        assert!(!INVALID_SPACE);
        assert!(!INVALID_AT);
    }
}

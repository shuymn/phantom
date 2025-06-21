/// Const utilities for core operations
/// These functions can be evaluated at compile time for constant inputs
/// Path separator as const
pub const PATH_SEPARATOR: char = '/';
pub const PATH_SEPARATOR_BYTE: u8 = b'/';

/// Check if a path component is valid (const version)
/// Disallows: empty, ".", "..", contains null bytes
pub const fn is_valid_path_component(component: &str) -> bool {
    let bytes = component.as_bytes();
    let len = bytes.len();

    // Empty component
    if len == 0 {
        return false;
    }

    // Single dot
    if len == 1 && bytes[0] == b'.' {
        return false;
    }

    // Double dot
    if len == 2 && bytes[0] == b'.' && bytes[1] == b'.' {
        return false;
    }

    // Check for null bytes
    let mut i = 0;
    while i < len {
        if bytes[i] == 0 {
            return false;
        }
        i += 1;
    }

    true
}

/// Validate path component and return it if valid
/// Returns empty string if component is invalid
pub const fn validate_path_component(component: &str) -> &str {
    if !is_valid_path_component(component) {
        return "";
    }
    component
}

/// Common file extensions
pub mod extensions {
    pub const GIT: &str = ".git";
    pub const LOCK: &str = ".lock";
    pub const TMP: &str = ".tmp";
    pub const BACKUP: &str = ".backup";
    pub const ORIG: &str = ".orig";
}

/// Check if a string ends with a suffix (const version)
pub const fn const_ends_with(s: &str, suffix: &str) -> bool {
    let s_bytes = s.as_bytes();
    let suffix_bytes = suffix.as_bytes();
    let s_len = s_bytes.len();
    let suffix_len = suffix_bytes.len();

    if s_len < suffix_len {
        return false;
    }

    let start = s_len - suffix_len;
    let mut i = 0;
    while i < suffix_len {
        if s_bytes[start + i] != suffix_bytes[i] {
            return false;
        }
        i += 1;
    }

    true
}

/// Check if a filename has a specific extension
pub const fn has_extension(filename: &str, ext: &str) -> bool {
    const_ends_with(filename, ext)
}

/// Environment variable names as constants
pub mod env_vars {
    pub const HOME: &str = "HOME";
    pub const USER: &str = "USER";
    pub const SHELL: &str = "SHELL";
    pub const PATH: &str = "PATH";
    pub const GIT_DIR: &str = "GIT_DIR";
    pub const GIT_WORK_TREE: &str = "GIT_WORK_TREE";
    pub const PHANTOM_DIR: &str = "PHANTOM_DIR";
    pub const EDITOR: &str = "EDITOR";
    pub const VISUAL: &str = "VISUAL";
}

/// Common directory names
pub mod dirs {
    pub const GIT: &str = ".git";
    pub const PHANTOM: &str = "phantom";
    pub const WORKTREES: &str = "worktrees";
    pub const HOOKS: &str = "hooks";
    pub const REFS: &str = "refs";
    pub const OBJECTS: &str = "objects";
}

/// File permissions as const (Unix)
pub mod permissions {
    pub const READABLE: u32 = 0o444;
    pub const WRITABLE: u32 = 0o222;
    pub const EXECUTABLE: u32 = 0o111;
    pub const USER_RWX: u32 = 0o700;
    pub const USER_RW: u32 = 0o600;
    pub const ALL_READ: u32 = 0o444;
    pub const DIR_DEFAULT: u32 = 0o755;
    pub const FILE_DEFAULT: u32 = 0o644;
}

/// Maximum values as constants
pub mod limits {
    pub const MAX_PATH_LENGTH: usize = 4096;
    pub const MAX_FILENAME_LENGTH: usize = 255;
    pub const MAX_SYMLINK_DEPTH: usize = 40;
    pub const MAX_COMMAND_LENGTH: usize = 32768;
}

/// Check if a character is a path separator
pub const fn is_path_separator(ch: char) -> bool {
    ch == PATH_SEPARATOR
}

/// Count occurrences of a character in a string (const version)
pub const fn count_char(s: &str, target: char) -> usize {
    let bytes = s.as_bytes();
    let target_byte = target as u8;
    let mut count = 0;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == target_byte {
            count += 1;
        }
        i += 1;
    }

    count
}

/// Get the depth of a path (number of separators)
pub const fn path_depth(path: &str) -> usize {
    count_char(path, PATH_SEPARATOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_component_validation() {
        const VALID1: bool = is_valid_path_component("file");
        const VALID2: bool = is_valid_path_component("file.txt");
        const VALID3: bool = is_valid_path_component("my-component");

        assert!(VALID1);
        assert!(VALID2);
        assert!(VALID3);

        const INVALID1: bool = is_valid_path_component("");
        const INVALID2: bool = is_valid_path_component(".");
        const INVALID3: bool = is_valid_path_component("..");

        assert!(!INVALID1);
        assert!(!INVALID2);
        assert!(!INVALID3);
    }

    #[test]
    fn test_const_ends_with() {
        const TEST1: bool = const_ends_with("file.txt", ".txt");
        const TEST2: bool = const_ends_with("file.txt", ".doc");
        const TEST3: bool = const_ends_with("test", "test");
        const TEST4: bool = const_ends_with("short", "longer");

        assert!(TEST1);
        assert!(!TEST2);
        assert!(TEST3);
        assert!(!TEST4);
    }

    #[test]
    fn test_has_extension() {
        const HAS_GIT: bool = has_extension("config.git", extensions::GIT);
        const HAS_LOCK: bool = has_extension("file.lock", extensions::LOCK);
        const NO_EXT: bool = has_extension("file", extensions::TMP);

        assert!(HAS_GIT);
        assert!(HAS_LOCK);
        assert!(!NO_EXT);
    }

    #[test]
    fn test_count_char() {
        const COUNT1: usize = count_char("a/b/c/d", '/');
        const COUNT2: usize = count_char("no-slashes", '/');
        const COUNT3: usize = count_char("///", '/');
        const COUNT4: usize = count_char("", '/');

        assert_eq!(COUNT1, 3);
        assert_eq!(COUNT2, 0);
        assert_eq!(COUNT3, 3);
        assert_eq!(COUNT4, 0);
    }

    #[test]
    fn test_path_depth() {
        const DEPTH1: usize = path_depth("/home/user/project");
        const DEPTH2: usize = path_depth("relative/path");
        const DEPTH3: usize = path_depth("file.txt");
        const DEPTH4: usize = path_depth("/");

        assert_eq!(DEPTH1, 3);
        assert_eq!(DEPTH2, 1);
        assert_eq!(DEPTH3, 0);
        assert_eq!(DEPTH4, 1);
    }

    #[test]
    fn test_constants() {
        // Verify some key constants
        assert_eq!(PATH_SEPARATOR, '/');
        assert_eq!(env_vars::HOME, "HOME");
        assert_eq!(dirs::GIT, ".git");
        assert_eq!(permissions::USER_RWX, 0o700);
        assert_eq!(limits::MAX_FILENAME_LENGTH, 255);
    }

    #[test]
    fn test_is_path_separator() {
        const IS_SEP1: bool = is_path_separator('/');
        const IS_SEP2: bool = is_path_separator('\\');
        const IS_SEP3: bool = is_path_separator('a');

        assert!(IS_SEP1);
        assert!(!IS_SEP2); // Not on Unix
        assert!(!IS_SEP3);
    }
}

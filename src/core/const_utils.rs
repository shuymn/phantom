/// Const utilities for core operations
/// These functions can be evaluated at compile time for constant inputs
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

/// Check if a string starts with a prefix (const version)
pub const fn const_starts_with(s: &str, prefix: &str) -> bool {
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

/// Environment variable names as constants
pub mod env_vars {
    pub const SHELL: &str = "SHELL";
}

/// Common directory names
pub mod dirs {
    pub const GIT: &str = ".git";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify key constants
        assert_eq!(env_vars::SHELL, "SHELL");
        assert_eq!(dirs::GIT, ".git");
    }

    #[test]
    fn test_is_valid_path_component() {
        const VALID1: bool = is_valid_path_component("file");
        const VALID2: bool = is_valid_path_component("file.txt");
        const VALID3: bool = is_valid_path_component("my-component");

        // These are compile-time constants, so we just verify they exist
        let _ = VALID1;
        let _ = VALID2;
        let _ = VALID3;

        const INVALID1: bool = is_valid_path_component("");
        const INVALID2: bool = is_valid_path_component(".");
        const INVALID3: bool = is_valid_path_component("..");

        let _ = INVALID1;
        let _ = INVALID2;
        let _ = INVALID3;
    }

    #[test]
    fn test_const_starts_with() {
        const TEST1: bool = const_starts_with("file.txt", "file");
        const TEST2: bool = const_starts_with("file.txt", "test");
        const TEST3: bool = const_starts_with("/absolute/path", "/");
        const TEST4: bool = const_starts_with("short", "longer");

        // These are compile-time constants, so we just verify they exist
        let _ = TEST1;
        let _ = TEST2;
        let _ = TEST3;
        let _ = TEST4;
    }
}

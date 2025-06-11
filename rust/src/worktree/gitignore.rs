use crate::Result;
use std::path::Path;
use tokio::fs;
use tracing::debug;

/// A gitignore pattern matcher
#[derive(Debug, Clone)]
pub struct GitignoreMatcher {
    patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
struct Pattern {
    pattern: String,
    is_negation: bool,
    is_directory: bool,
    anchored: bool,
}

impl GitignoreMatcher {
    /// Create a new empty matcher
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    /// Load patterns from a .gitignore file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Ok(Self::from_content(&content))
    }

    /// Create a matcher from gitignore content
    pub fn from_content(content: &str) -> Self {
        let patterns = content
            .lines()
            .filter_map(|line| {
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }

                // Check for negation
                let (is_negation, line) = match line.strip_prefix('!') {
                    Some(stripped) => (true, stripped),
                    None => (false, line),
                };

                // Check if pattern is for directories only
                let (is_directory, pattern) = if line.ends_with('/') {
                    (true, line.trim_end_matches('/'))
                } else {
                    (false, line)
                };

                // Check if pattern is anchored to root
                let anchored = pattern.starts_with('/');
                let pattern = pattern.trim_start_matches('/').to_string();

                Some(Pattern { pattern, is_negation, is_directory, anchored })
            })
            .collect();

        Self { patterns }
    }

    /// Add patterns from another matcher
    pub fn extend(&mut self, other: &GitignoreMatcher) {
        self.patterns.extend(other.patterns.clone());
    }

    /// Check if a path should be ignored
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let path_str = path.to_string_lossy();
        let mut ignored = false;

        for pattern in &self.patterns {
            // Skip directory-only patterns if checking a file
            if pattern.is_directory && !is_dir {
                continue;
            }

            if self.matches_pattern(&path_str, pattern) {
                ignored = !pattern.is_negation;
            }
        }

        ignored
    }

    /// Check if a path matches a specific pattern
    fn matches_pattern(&self, path: &str, pattern: &Pattern) -> bool {
        let pattern_str = &pattern.pattern;

        // Handle anchored patterns
        if pattern.anchored {
            self.glob_match(path, pattern_str)
        } else {
            // Non-anchored patterns can match anywhere in the path
            if self.glob_match(path, pattern_str) {
                return true;
            }

            // Also check if pattern matches any suffix of the path
            let parts: Vec<&str> = path.split('/').collect();
            for i in 1..parts.len() {
                let suffix = parts[i..].join("/");
                if self.glob_match(&suffix, pattern_str) {
                    return true;
                }
            }

            false
        }
    }

    /// Simple glob pattern matching
    fn glob_match(&self, text: &str, pattern: &str) -> bool {
        // This is a simplified glob matcher
        // In production, consider using the `glob` or `globset` crate

        if pattern == "*" {
            return true;
        }

        // Handle patterns with wildcards
        if pattern.contains('*') {
            // Convert pattern to regex-like matching
            let parts: Vec<&str> = pattern.split('*').collect();

            if parts.is_empty() {
                return true;
            }

            let mut text_pos = 0;

            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }

                if i == 0 && !pattern.starts_with('*') {
                    // Pattern must start with this part
                    if !text.starts_with(part) {
                        return false;
                    }
                    text_pos = part.len();
                } else if i == parts.len() - 1 && !pattern.ends_with('*') {
                    // Pattern must end with this part
                    return text.ends_with(part);
                } else {
                    // Find this part in the remaining text
                    if let Some(pos) = text[text_pos..].find(part) {
                        text_pos += pos + part.len();
                    } else {
                        return false;
                    }
                }
            }

            true
        } else {
            // Exact match
            text == pattern
        }
    }
}

impl Default for GitignoreMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Load all gitignore patterns from a directory hierarchy
pub async fn load_gitignore_hierarchy(start_dir: &Path) -> Result<GitignoreMatcher> {
    let mut matcher = GitignoreMatcher::new();
    let mut current = start_dir;

    // Walk up the directory hierarchy
    loop {
        let gitignore_path = current.join(".gitignore");
        if let Ok(content) = fs::read_to_string(&gitignore_path).await {
            debug!("Loading .gitignore from {}", gitignore_path.display());
            let local_matcher = GitignoreMatcher::from_content(&content);
            matcher.extend(&local_matcher);
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Ok(matcher)
}

/// Default patterns to always ignore
pub fn default_ignore_patterns() -> GitignoreMatcher {
    GitignoreMatcher::from_content(
        r#"
# Git directory
.git/

# Common build directories
target/
build/
dist/
out/

# Dependencies
node_modules/
vendor/

# IDE files
.idea/
.vscode/
*.swp
*.swo
*~

# OS files
.DS_Store
Thumbs.db

# Logs
*.log
"#,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_patterns() {
        let content = r#"
# Comment
*.log
!important.log
/src/
build/
*.tmp
"#;

        let matcher = GitignoreMatcher::from_content(content);
        assert_eq!(matcher.patterns.len(), 5);

        assert_eq!(matcher.patterns[0].pattern, "*.log");
        assert!(!matcher.patterns[0].is_negation);

        assert_eq!(matcher.patterns[1].pattern, "important.log");
        assert!(matcher.patterns[1].is_negation);

        assert_eq!(matcher.patterns[2].pattern, "src");
        assert!(matcher.patterns[2].is_directory);
        assert!(matcher.patterns[2].anchored);
    }

    #[test]
    fn test_glob_match() {
        let matcher = GitignoreMatcher::new();

        // Exact match
        assert!(matcher.glob_match("test.log", "test.log"));
        assert!(!matcher.glob_match("test.txt", "test.log"));

        // Wildcard at end
        assert!(matcher.glob_match("test.log", "*.log"));
        assert!(matcher.glob_match("debug.log", "*.log"));
        assert!(!matcher.glob_match("test.txt", "*.log"));

        // Wildcard at beginning
        assert!(matcher.glob_match("test.log", "test.*"));
        assert!(matcher.glob_match("test.txt", "test.*"));
        assert!(!matcher.glob_match("debug.log", "test.*"));

        // Multiple wildcards
        assert!(matcher.glob_match("test.backup.log", "*.backup.*"));
        assert!(!matcher.glob_match("test.log", "*.backup.*"));
    }

    #[test]
    fn test_is_ignored() {
        let content = r#"
*.log
!important.log
build/
/src/temp/
"#;

        let matcher = GitignoreMatcher::from_content(content);

        // Regular patterns
        assert!(matcher.is_ignored(Path::new("debug.log"), false));
        assert!(matcher.is_ignored(Path::new("path/to/debug.log"), false));

        // Negation
        assert!(!matcher.is_ignored(Path::new("important.log"), false));

        // Directory patterns
        assert!(matcher.is_ignored(Path::new("build"), true));
        assert!(matcher.is_ignored(Path::new("path/to/build"), true));
        assert!(!matcher.is_ignored(Path::new("build"), false)); // Not a directory

        // Anchored patterns
        assert!(matcher.is_ignored(Path::new("src/temp"), true));
        assert!(!matcher.is_ignored(Path::new("other/src/temp"), true));
    }

    #[test]
    fn test_default_patterns() {
        let matcher = default_ignore_patterns();

        assert!(matcher.is_ignored(Path::new(".git"), true));
        assert!(matcher.is_ignored(Path::new("node_modules"), true));
        assert!(matcher.is_ignored(Path::new(".DS_Store"), false));
        assert!(matcher.is_ignored(Path::new("test.log"), false));
    }

    #[tokio::test]
    async fn test_load_from_file() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        fs::write(&gitignore_path, "*.log\n!important.log\n").await.unwrap();

        let matcher = GitignoreMatcher::load_from_file(&gitignore_path).await.unwrap();
        assert_eq!(matcher.patterns.len(), 2);
    }
}

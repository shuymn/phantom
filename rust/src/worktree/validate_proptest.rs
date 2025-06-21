#[cfg(test)]
mod tests {
    use super::super::validate::*;
    use proptest::prelude::*;

    // Property: Valid names should always pass validation
    proptest! {
        #[test]
        fn valid_names_always_pass(
            s in "[a-zA-Z0-9][a-zA-Z0-9._/-]{0,99}"
        ) {
            // Skip patterns with consecutive dots as they're invalid
            prop_assume!(!s.contains(".."));

            let result = validate_worktree_name(&s);
            prop_assert!(result.is_ok());
        }
    }

    // Property: Names with invalid characters should always fail
    proptest! {
        #[test]
        fn names_with_invalid_chars_fail(
            prefix in "[a-zA-Z0-9]{1,10}",
            invalid_char in "[^a-zA-Z0-9._/-]".prop_map(|s| s.chars().next().unwrap()),
            suffix in "[a-zA-Z0-9._/-]{0,10}"
        ) {
            // Skip if the invalid char is actually valid (edge case with regex)
            prop_assume!(invalid_char != '.' && invalid_char != '-' && invalid_char != '_' && invalid_char != '/');

            let name = format!("{}{}{}", prefix, invalid_char, suffix);
            let result = validate_worktree_name(&name);
            prop_assert!(result.is_err());
        }
    }

    // Property: Empty names should always fail
    proptest! {
        #[test]
        fn empty_names_fail(
            spaces in " {0,10}"
        ) {
            let result = validate_worktree_name(&spaces);
            prop_assert!(result.is_err());
        }
    }

    // Property: Names with consecutive dots should fail
    proptest! {
        #[test]
        fn names_with_consecutive_dots_fail(
            prefix in "[a-zA-Z0-9]{0,10}",
            suffix in "[a-zA-Z0-9._/-]{0,10}"
        ) {
            let name = format!("{}..{}", prefix, suffix);
            let result = validate_worktree_name(&name);
            prop_assert!(result.is_err());
        }
    }

    // Property: Names up to MAX_WORKTREE_NAME_LENGTH should be accepted
    proptest! {
        #[test]
        fn long_names_accepted(
            s in "[a-zA-Z0-9][a-zA-Z0-9._/-]{100,254}"
        ) {
            // Skip patterns with consecutive dots
            prop_assume!(!s.contains(".."));
            // Ensure we don't exceed the maximum length
            prop_assume!(s.len() <= crate::worktree::const_validate::MAX_WORKTREE_NAME_LENGTH);

            let result = validate_worktree_name(&s);
            prop_assert!(result.is_ok());
        }
    }

    // Property: Names exceeding MAX_WORKTREE_NAME_LENGTH should fail
    proptest! {
        #[test]
        fn too_long_names_fail(
            s in "[a-zA-Z0-9][a-zA-Z0-9._/-]{255,300}"
        ) {
            // Skip patterns with consecutive dots
            prop_assume!(!s.contains(".."));
            // Ensure we exceed the maximum length
            prop_assume!(s.len() > crate::worktree::const_validate::MAX_WORKTREE_NAME_LENGTH);

            let result = validate_worktree_name(&s);
            prop_assert!(result.is_err());
        }
    }

    // Property: Names with slashes are valid (for branch-like names)
    proptest! {
        #[test]
        fn names_with_slashes_valid(
            parts in prop::collection::vec("[a-zA-Z0-9._-]{1,20}", 1..5)
        ) {
            let name = parts.join("/");
            // Skip if any part is empty or contains consecutive dots
            prop_assume!(!name.contains("..") && !name.contains("//"));

            let result = validate_worktree_name(&name);
            prop_assert!(result.is_ok());
        }
    }

    // Property: Common branch name patterns should be valid
    proptest! {
        #[test]
        fn common_branch_patterns_valid(
            prefix in prop::sample::select(&["feature", "bugfix", "release", "hotfix"]),
            issue_num in 1u32..10000u32,
            description in "[a-z0-9-]{1,30}"
        ) {
            let name = format!("{}/{}-{}", prefix, issue_num, description);
            let result = validate_worktree_name(&name);
            prop_assert!(result.is_ok());
        }
    }

    // Property: Semantic version patterns should be valid
    proptest! {
        #[test]
        fn version_patterns_valid(
            major in 0u32..100u32,
            minor in 0u32..100u32,
            patch in 0u32..100u32,
            prerelease in prop::option::of("[a-zA-Z0-9.-]{1,20}")
        ) {
            let name = match prerelease {
                Some(pre) => format!("v{}.{}.{}-{}", major, minor, patch, pre),
                None => format!("v{}.{}.{}", major, minor, patch),
            };

            // Skip if prerelease contains consecutive dots
            prop_assume!(!name.contains(".."));

            let result = validate_worktree_name(&name);
            prop_assert!(result.is_ok());
        }
    }
}

use jag_core::types::TaskId;

/// Generates a hybrid branch name following the pattern: `jag/{slug}-{short-task-id}`.
pub fn generate_branch_name(
    prompt: &str,
    task_id: &TaskId,
    prefix: &str,
    max_slug_words: usize,
) -> String {
    // 1. Convert prompt to a slug
    let slug: String = prompt
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .take(max_slug_words)
        .collect::<Vec<_>>()
        .join("-");

    // 2. Extract short version of task ID (first 6 chars of UUID)
    let short_id = task_id.to_string();
    let short_id = &short_id[..6.min(short_id.len())];

    if slug.is_empty() {
        format!("{}/mission-{}", prefix, short_id)
    } else {
        format!("{}/{}-{}", prefix, slug, short_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_branch_name_generation() {
        let task_id = TaskId(Uuid::parse_str("7f2a1b3c-4d5e-6f7a-8b9c-0d1e2f3a4b5c").unwrap());
        let prompt = "Add user login form with OAuth";
        
        // Default settings
        let name = generate_branch_name(prompt, &task_id, "jag", 4);
        assert_eq!(name, "jag/add-user-login-form-7f2a1b");
    }

    #[test]
    fn test_branch_name_empty_prompt() {
        let task_id = TaskId(Uuid::parse_str("7f2a1b3c-4d5e-6f7a-8b9c-0d1e2f3a4b5c").unwrap());
        let prompt = "!!!"; // Becomes empty slug
        
        let name = generate_branch_name(prompt, &task_id, "jag", 4);
        assert_eq!(name, "jag/mission-7f2a1b");
    }
}

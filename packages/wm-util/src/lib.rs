/// Unescape `\n` and `\t` in text input fields
pub fn unescape_text(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\t", "\t")
}

/// Truncate a string to N chars, appending "..." if truncated
pub fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // Use chars() to safely handle multi-byte UTF-8 boundaries
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Convert a title to a filesystem-safe slug
pub fn slugify(title: &str) -> String {
    let mut slug = title.to_lowercase();
    slug = slug.replace(|c: char| !c.is_alphanumeric() && c != '-', "-");
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

/// Format seconds as "2h30m" or "45s"
pub fn format_duration(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_text() {
        assert_eq!(unescape_text("line1\\nline2"), "line1\nline2");
        assert_eq!(unescape_text("col1\\tcol2"), "col1\tcol2");
        assert_eq!(unescape_text("no escapes"), "no escapes");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert!(truncate_str("hello world this is long", 10).ends_with("..."));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Auth & Security!"), "auth-security");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(130), "2m10s");
        assert_eq!(format_duration(3700), "1h1m");
    }
}

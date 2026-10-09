//! Minimal glob matching for repo-relative paths: `**` (any dirs), `*` (within a segment),
//! `?` (one char). Enough for `[text].files` patterns; avoids a dependency.

use regex::Regex;

pub fn to_regex(pattern: &str) -> Regex {
    let mut re = String::from("(?s)^");
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' if chars.get(i + 1) == Some(&'*') => {
                // `**/` matches zero or more directories; a trailing `**` matches everything.
                if chars.get(i + 2) == Some(&'/') {
                    re.push_str("(?:.*/)?");
                    i += 3;
                } else {
                    re.push_str(".*");
                    i += 2;
                }
                continue;
            }
            '*' => re.push_str("[^/]*"),
            '?' => re.push_str("[^/]"),
            c => re.push_str(&regex::escape(&c.to_string())),
        }
        i += 1;
    }
    re.push('$');
    Regex::new(&re).expect("glob always compiles")
}

pub fn matches_any(patterns: &[Regex], path: &str) -> bool {
    patterns.iter().any(|p| p.is_match(path))
}

#[cfg(test)]
mod tests {
    use super::to_regex;

    #[test]
    fn globs() {
        let r = to_regex("App/**/*.swift");
        assert!(r.is_match("App/a.swift"));
        assert!(r.is_match("App/x/y/a.swift"));
        assert!(!r.is_match("Other/a.swift"));
        assert!(!r.is_match("App/a.swiftx"));
        assert!(to_regex("*.md").is_match("README.md"));
        assert!(!to_regex("*.md").is_match("docs/README.md"));
        assert!(to_regex("docs/**").is_match("docs/a/b.txt"));
        assert!(to_regex("a?.txt").is_match("ab.txt"));
        assert!(!to_regex("a.txt").is_match("abtxt"));
    }
}

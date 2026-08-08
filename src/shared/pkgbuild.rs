use std::collections::HashSet;

/// Extract literal package identifiers in lookup order:
/// pkgbase, then pkgname, then _pkgname.
pub fn package_name_candidates(content: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    for key in ["pkgbase", "pkgname", "_pkgname"] {
        let Some(value) = assignment_value(content, key) else {
            continue;
        };

        for candidate in literal_words(&value) {
            if !candidate.is_empty() && !candidate.contains('$') && seen.insert(candidate.clone()) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

fn assignment_value(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();

    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        let Some(rest) = trimmed.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(mut value) = rest.strip_prefix('=').map(str::trim_start) else {
            continue;
        };

        if !value.starts_with('(') || parentheses_balanced(value) {
            return Some(value.to_string());
        }

        let mut collected = value.to_string();
        for next in lines.by_ref() {
            collected.push('\n');
            collected.push_str(next);
            if parentheses_balanced(&collected) {
                break;
            }
        }
        value = &collected;
        return Some(value.to_string());
    }

    None
}

fn parentheses_balanced(value: &str) -> bool {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;

    for ch in value.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if matches!(ch, '\'' | '"') {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            }
            continue;
        }
        if quote.is_none() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
        }
    }

    depth <= 0
}

fn literal_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut comment = false;

    for ch in value.chars() {
        if comment {
            if ch == '\n' {
                comment = false;
            }
            continue;
        }
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if matches!(ch, '\'' | '"') {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            } else {
                current.push(ch);
            }
            continue;
        }
        if quote.is_none() {
            if ch == '#' {
                push_word(&mut words, &mut current);
                comment = true;
                continue;
            }
            if ch.is_whitespace() || matches!(ch, '(' | ')') {
                push_word(&mut words, &mut current);
                continue;
            }
        }
        current.push(ch);
    }
    push_word(&mut words, &mut current);

    words
}

fn push_word(words: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        words.push(std::mem::take(current));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_pkgbase_then_pkgname_then_private_name() {
        let content = r#"
_pkgname=notepad-plus-plus
pkgname=notepad++
pkgbase=notepadpp
"#;

        assert_eq!(
            package_name_candidates(content),
            vec!["notepadpp", "notepad++", "notepad-plus-plus"]
        );
    }

    #[test]
    fn falls_back_when_higher_priority_names_are_missing() {
        assert_eq!(
            package_name_candidates("pkgname=foo\n_pkgname=bar"),
            vec!["foo", "bar"]
        );
        assert_eq!(package_name_candidates("_pkgname=bar"), vec!["bar"]);
    }

    #[test]
    fn supports_quoted_and_multiline_pkgname_arrays() {
        let content = r#"
pkgbase='suite'
pkgname=(
  "suite-cli"
  suite-gui # desktop package
)
"#;

        assert_eq!(
            package_name_candidates(content),
            vec!["suite", "suite-cli", "suite-gui"]
        );
    }

    #[test]
    fn ignores_comments_dynamic_values_and_similar_keys() {
        let content = r#"
# pkgbase=wrong
other_pkgname=wrong
pkgbase=$dynamic
pkgname=real
_pkgname=private
"#;

        assert_eq!(package_name_candidates(content), vec!["real", "private"]);
    }

    #[test]
    fn removes_duplicate_candidates() {
        let content = "pkgbase=same\npkgname=same\n_pkgname=same";

        assert_eq!(package_name_candidates(content), vec!["same"]);
    }
}

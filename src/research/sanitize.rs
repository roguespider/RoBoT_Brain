pub fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

pub fn strip_control_chars(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect()
}

pub fn cap_and_truncate(
    sources: Vec<crate::research::provider::SearchResult>,
    max: usize,
) -> (Vec<crate::research::provider::SearchResult>, String) {
    let total = sources.len();
    let truncated = sources.into_iter().take(max).collect::<Vec<_>>();
    let remaining = total.saturating_sub(max);
    let marker = if remaining > 0 {
        format!("and {} more", remaining)
    } else {
        String::new()
    };
    (truncated, marker)
}

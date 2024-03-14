use crate::MatchStyle;
use owo_colors::Style;
use regex::Regex;

/// Style for `diff -u` output.
pub fn diff() -> Vec<MatchStyle> {
    vec![
        MatchStyle {
            pattern: Regex::new(r"^\+.*$").unwrap(),
            style: Style::new().bright_green(),
        },
        MatchStyle {
            pattern: Regex::new(r"^\-.*$").unwrap(),
            style: Style::new().bright_red(),
        },
        MatchStyle {
            pattern: Regex::new(r"^@@.*$").unwrap(),
            style: Style::new().yellow(),
        },
    ]
}

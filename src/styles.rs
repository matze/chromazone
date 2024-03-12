use crate::MatchStyle;
use ansi_term::{Color, Style};
use regex::Regex;

/// Style for `diff -u` output.
pub fn diff() -> Vec<MatchStyle> {
    vec![
        MatchStyle {
            expr: Regex::new(r"^\+.*$").unwrap(),
            style: Style::new().fg(Color::Green),
        },
        MatchStyle {
            expr: Regex::new(r"^\-.*$").unwrap(),
            style: Style::new().fg(Color::Red),
        },
        MatchStyle {
            expr: Regex::new(r"^@@.*$").unwrap(),
            style: Style::new().fg(Color::Yellow),
        },
    ]
}

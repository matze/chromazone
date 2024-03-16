//! Functionality to read styles from a configuration file.
//!
//! # Location
//!
//! If a named style is requested, the style configuration file is looked up according ot the XDG
//! base dir spec, i.e. in `$XDG_CONFIG_HOME/chromazone/chromazone.styles` or
//! `$HOME/.config/chromazone/chromazone.styles` if `XDG_CONFIG_HOME` is not set.
//!
//! # Format
//!
//! The format of the configuration file is similar to INI or TOML with sections describing a
//! specific named style. However, the contents of a section are in fact lines beginning with a
//! regex match pattern, whitespace and a list of style colors and effects
//!
//! ```
//! [diff]
//! "^@@.*@@$" yellow
//! "^-.*" red
//! "^+.*" green
//! ```

use crate::{Description, MatchStyle};
use regex::Regex;
use std::path::PathBuf;

/// Find a list of [`MatchStyle`] in the `config` string under the `style` section.
fn find_style(config: &str, style: &str) -> Result<Vec<MatchStyle>, String> {
    let section = Regex::new(r"^\s*\[(\w+)\]\s*$").expect("creating section pattern");
    let match_style = Regex::new(r#"^\s*"(.*)"\s*(.*)$"#).expect("creating match style pattern");
    let mut result = Vec::new();
    let mut append = false;

    for line in config.lines() {
        if append {
            if let Some(captures) = match_style.captures(line) {
                let pattern = Regex::new(&captures[1]).map_err(|err| err.to_string())?;
                let style = Description(&captures[2]).try_into()?;
                result.push(MatchStyle { pattern, style });
                continue;
            }
        }

        if let Some(captures) = section.captures(line) {
            append = &captures[1] == style;
        }
    }

    Ok(result)
}

pub fn read_style_from_config(style: &str) -> Result<Vec<MatchStyle>, String> {
    let mut path = match (std::env::var("XDG_CONFIG_HOME"), std::env::var("HOME")) {
        (Ok(config_home), _) => PathBuf::from(config_home),
        (_, Ok(home)) => PathBuf::from(home).join(".config"),
        (_, _) => {
            return Ok(Vec::new());
        }
    };

    path.push("chromazone");
    path.push("chromazone.styles");

    if path.exists() && path.is_file() {
        let config = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
        find_style(&config, style)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"[foo]
"hello" green,bold

[qux]
"world" yellow,underline
"foo" red
"#;

    #[test]
    fn find_none() {
        assert!(find_style(CONFIG, "bar").unwrap().is_empty());
    }

    #[test]
    fn find_all_styles() {
        let styles = find_style(CONFIG, "foo").unwrap();
        assert_eq!(styles.len(), 1);

        let styles = find_style(CONFIG, "qux").unwrap();
        assert_eq!(styles.len(), 2);
    }
}

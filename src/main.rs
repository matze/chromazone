use ansi_term::Style;
use regex::Regex;
use std::process::ExitCode;

mod styles;

/// Style for a matched region.
struct MatchStyle {
    /// Regular expression whose match will be styled with `style`.
    expr: Regex,
    /// Text style for the match.
    style: Style,
}

/// Region inside a line which is either unmatched and printed verbatim or matched and printed with
/// a style applied.
enum Region<'input, 'style> {
    /// Text region has no match pattern and to be printed verbatim.
    Unmatched { text: &'input str },
    /// Text region matched and is to be styled with `style`.
    Matched {
        text: &'input str,
        style: &'style Style,
    },
}

/// Iterator of matched and unmatched regions over `text`.
struct Regions<'input, 'style> {
    /// Remaining text to be matched.
    text: &'input str,
    /// Available match expressions and styles.
    styles: &'style [MatchStyle],
}

impl<'input, 'style> Iterator for Regions<'input, 'style> {
    type Item = Region<'input, 'style>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }

        // TODO: fix inefficiency of discarding the matches.
        match self
            .styles
            .iter()
            .find_map(|style| style.expr.find(self.text).map(|m| (m, style)))
        {
            None => {
                let text = self.text;
                self.text = &self.text[self.text.len()..];
                Some(Region::Unmatched { text })
            }
            Some((m, MatchStyle { expr: _, style })) => {
                let start = m.start();

                if start > 0 {
                    let text = &self.text[..start];
                    self.text = &self.text[start..];

                    Some(Region::Unmatched { text })
                } else {
                    let end = m.end();
                    let text = &self.text[..end];
                    self.text = &self.text[end..];

                    Some(Region::Matched { text, style })
                }
            }
        }
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);

    let styles = match (args.next(), args.next()) {
        (Some(opt), None) => {
            if opt == "--help" || opt == "-h" {
                println!("Usage: <prog> | cz [--style <style>] [-h|--help]");
                return ExitCode::SUCCESS;
            }

            vec![]
        }
        (Some(opt), Some(arg)) => {
            if opt == "--style" || opt == "-s" {
                match arg.as_str() {
                    "diff" => styles::diff(),
                    _ => {
                        eprintln!("Error: unknown style '{arg}'");
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                vec![]
            }
        }
        _ => {
            vec![]
        }
    };

    for line in std::io::stdin().lines() {
        let text = line.unwrap();
        let regions = Regions {
            text: &text,
            styles: &styles,
        };

        for region in regions {
            match region {
                Region::Unmatched { text } => print!("{text}"),
                Region::Matched { text, style } => print!("{}", style.paint(text)),
            }
        }

        println!();
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_match() {
        let expr = Regex::new("needle").unwrap();
        let styles = &[MatchStyle {
            expr,
            style: Style::new(),
        }];

        let mut regions = Regions {
            text: "haystack",
            styles,
        };

        assert!(matches!(
            regions.next(),
            Some(Region::Unmatched { text: "haystack" })
        ));

        assert!(regions.next().is_none());
    }

    #[test]
    fn match_in_the_middle() {
        let expr = Regex::new("needle").unwrap();
        let styles = &[MatchStyle {
            expr,
            style: Style::new(),
        }];

        let mut regions = Regions {
            text: "a needle in the haystack",
            styles,
        };

        assert!(matches!(
            regions.next(),
            Some(Region::Unmatched { text: "a " })
        ));

        assert!(matches!(
            regions.next(),
            Some(Region::Matched { text: "needle", .. })
        ));

        assert!(matches!(
            regions.next(),
            Some(Region::Unmatched {
                text: " in the haystack"
            })
        ));

        assert!(regions.next().is_none());
    }
}

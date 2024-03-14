use owo_colors::{OwoColorize, Style};
use regex::Regex;
use std::process::ExitCode;

mod styles;

/// Style for a matched region.
struct MatchStyle {
    /// Regular expression pattern whose match will be styled with `style`.
    pattern: Regex,
    /// Text style for the match.
    style: Style,
}

/// String description of a style.
///
/// Description consists of a comma-separated list of colors and effects.
struct Description<'input>(&'input str);

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
    /// Previous match.
    previous: Option<(&'input str, &'style Style)>,
}

/// Parsed command line options.
#[derive(Default)]
struct Opts {
    /// `true` if help is requested.
    help: bool,
    /// Loaded match style.
    styles: Vec<MatchStyle>,
}

impl<'input, 'style> Regions<'input, 'style> {
    fn new(text: &'input str, styles: &'style [MatchStyle]) -> Self {
        Self {
            text,
            styles,
            previous: None,
        }
    }
}

impl<'input, 'style> Iterator for Regions<'input, 'style> {
    type Item = Region<'input, 'style>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }

        if let Some((text, style)) = self.previous.take() {
            return Some(Region::Matched { text, style });
        }

        match self
            .styles
            .iter()
            .find_map(|style| style.pattern.find(self.text).map(|m| (m, style)))
        {
            None => {
                let text = self.text;
                self.text = &self.text[self.text.len()..];
                Some(Region::Unmatched { text })
            }
            Some((m, MatchStyle { pattern: _, style })) => {
                let start = m.start();

                if start > 0 {
                    // The match is not at the beginning, so store it, return unmatched text now
                    // and the match in the next iteration.
                    let text = &self.text[..start];
                    self.text = &self.text[m.end()..];
                    self.previous = Some((m.as_str(), style));

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

impl Opts {
    fn parse() -> Result<Self, String> {
        let mut opts = Self::default();
        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            if arg == "--help" || arg == "-h" {
                opts.help = true;
            }

            if arg == "--style" || arg == "-s" {
                match args.next() {
                    None => {
                        return Err("expected style after --style/-s".into());
                    }
                    Some(name) => {
                        if name == "diff" {
                            opts.styles = styles::diff();
                        } else {
                            return Err(format!("unknown style '{}'", name.yellow().bold()));
                        }
                    }
                }
            }

            if arg == "--match" || arg == "-m" {
                match (args.next(), args.next()) {
                    (Some(pattern), Some(description)) => {
                        let pattern = Regex::new(&pattern).map_err(|err| err.to_string())?;
                        let style = Description(&description).try_into()?;
                        opts.styles = vec![MatchStyle { pattern, style }];
                    }
                    _ => return Err("expected pattern and style after --match/-m".into()),
                }
            }
        }

        Ok(opts)
    }
}

impl<'input> TryFrom<Description<'input>> for Style {
    type Error = String;

    fn try_from(value: Description<'input>) -> Result<Self, Self::Error> {
        let mut style = Style::new();

        for part in value.0.split(',') {
            match part.trim() {
                "black" => style = style.black(),
                "blue" => style = style.blue(),
                "cyan" => style = style.cyan(),
                "green" => style = style.green(),
                "magenta" => style = style.magenta(),
                "purple" => style = style.magenta(),
                "red" => style = style.red(),
                "white" => style = style.white(),
                "yellow" => style = style.yellow(),
                "bold" => style = style.bold(),
                "italic" => style = style.italic(),
                "strike" => style = style.strikethrough(),
                "underline" => style = style.underline(),
                _ => return Err(format!("unknown style part '{}'", part.yellow().bold())),
            }
        }

        Ok(style)
    }
}

fn try_main() -> Result<(), String> {
    let opts = Opts::parse()?;

    if opts.help {
        println!("Usage: <prog> | cz [--style <style>] [-h|--help]");
        return Ok(());
    }

    for line in std::io::stdin().lines() {
        let text = line.map_err(|err| err.to_string())?;

        for region in Regions::new(&text, &opts.styles) {
            match region {
                Region::Unmatched { text } => print!("{text}"),
                Region::Matched { text, style } => print!("{}", style.style(text)),
            }
        }

        println!();
    }

    Ok(())
}

fn main() -> ExitCode {
    match try_main() {
        Err(err) => {
            eprintln!("{}: {err}", "error".red().bold());
            ExitCode::FAILURE
        }
        Ok(()) => ExitCode::SUCCESS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_match() {
        let pattern = Regex::new("needle").unwrap();
        let styles = &[MatchStyle {
            pattern,
            style: Style::new(),
        }];

        let mut regions = Regions::new("haystack", styles);

        assert!(matches!(
            regions.next(),
            Some(Region::Unmatched { text: "haystack" })
        ));

        assert!(regions.next().is_none());
    }

    #[test]
    fn match_in_the_middle() {
        let pattern = Regex::new("needle").unwrap();
        let styles = &[MatchStyle {
            pattern,
            style: Style::new(),
        }];

        let mut regions = Regions::new("a needle in the haystack", styles);

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

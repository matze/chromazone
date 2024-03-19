use owo_colors::{OwoColorize, Style};
use regex::Regex;
use std::process::ExitCode;

mod config;

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

/// List of match styles.
struct MatchStyles<'style>(&'style [MatchStyle]);

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
    styles: &'style MatchStyles<'style>,
    /// Previous match.
    previous: Option<(&'input str, &'style Style)>,
}

/// Parser to match regions over lines.
struct Parser<'style> {
    /// Available match expressions and styles.
    styles: MatchStyles<'style>,
}

/// Parsed command line options.
#[derive(Default)]
struct Opts {
    /// `true` if help is requested.
    help: bool,
    /// Loaded match style.
    styles: Vec<MatchStyle>,
}

impl<'style> Parser<'style> {
    /// Create new [`Parser`] given the `styles` match patterns.
    fn new(styles: &'style [MatchStyle]) -> Self {
        Self {
            styles: MatchStyles::new(styles),
        }
    }

    /// Return [`Regions`] iterator over matched and umatched regions found in `text`.
    fn regions<'input>(&'style self, text: &'input str) -> Regions<'input, 'style> {
        Regions {
            text,
            styles: &self.styles,
            previous: None,
        }
    }
}

impl<'style> MatchStyles<'style> {
    /// Create a new [`MatchStyles`] object.
    fn new(styles: &'style [MatchStyle]) -> Self {
        Self(styles)
    }

    /// Find a match in `text` and the corresponding style or `None`.
    fn find_match<'input>(
        &self,
        text: &'input str,
    ) -> Option<(regex::Match<'input>, &'style Style)> {
        self.0
            .iter()
            .filter_map(|style| style.pattern.find(text).map(|m| (m, &style.style)))
            .min_by(|x, y| x.0.start().cmp(&y.0.start()))
    }
}

impl<'input, 'style> Iterator for Regions<'input, 'style> {
    type Item = Region<'input, 'style>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((text, style)) = self.previous.take() {
            return Some(Region::Matched { text, style });
        }

        if self.text.is_empty() {
            return None;
        }

        match self.styles.find_match(self.text) {
            None => {
                let text = self.text;
                self.text = &self.text[self.text.len()..];
                Some(Region::Unmatched { text })
            }
            Some((m, style)) => {
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
                        opts.styles
                            .append(&mut config::read_style_from_config(&name)?);
                    }
                }
            }

            if arg == "--match" || arg == "-m" {
                match (args.next(), args.next()) {
                    (Some(pattern), Some(description)) => {
                        let pattern = Regex::new(&pattern).map_err(|err| err.to_string())?;
                        let style = Description(&description).try_into()?;
                        opts.styles.push(MatchStyle { pattern, style });
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
                "b:black" => style = style.on_black(),
                "blue" => style = style.blue(),
                "b:blue" => style = style.on_blue(),
                "cyan" => style = style.cyan(),
                "b:cyan" => style = style.on_cyan(),
                "green" => style = style.green(),
                "b:green" => style = style.on_green(),
                "magenta" => style = style.magenta(),
                "b:magenta" => style = style.on_magenta(),
                "purple" => style = style.purple(),
                "b:purple" => style = style.on_purple(),
                "red" => style = style.red(),
                "b:red" => style = style.on_red(),
                "white" => style = style.white(),
                "b:white" => style = style.on_white(),
                "yellow" => style = style.yellow(),
                "b:yellow" => style = style.on_yellow(),
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

fn read_line(buf: &mut String) -> Result<usize, String> {
    std::io::stdin()
        .read_line(buf)
        .map_err(|err| err.to_string())
}

fn try_main() -> Result<(), String> {
    let opts = Opts::parse()?;

    if opts.help {
        println!(
            "{}: <prog> | {} [--style <style>] [-m|--match <pattern> <description>] [-h|--help]",
            "Usage".green().bold(),
            "cz".green().bold()
        );
        return Ok(());
    }

    let mut buf = String::new();
    let parser = Parser::new(&opts.styles);

    while read_line(&mut buf)? > 0 {
        let line = &buf[..buf.len() - 1];

        for region in parser.regions(line) {
            match region {
                Region::Unmatched { text } => print!("{text}"),
                Region::Matched { text, style } => print!("{}", style.style(text)),
            }
        }

        println!();
        buf.clear();
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

        let parser = Parser::new(styles);
        let mut regions = parser.regions("haystack");

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

        let parser = Parser::new(styles);
        let mut regions = parser.regions("a needle in the haystack");

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

    #[test]
    fn do_not_lose_text() {
        let styles = &[
            MatchStyle {
                pattern: Regex::new("foo").unwrap(),
                style: Style::new(),
            },
            MatchStyle {
                pattern: Regex::new("bar").unwrap(),
                style: Style::new(),
            },
        ];

        let parser = Parser::new(styles);
        let mut regions = parser.regions("foo bar");

        assert!(matches!(
            regions.next(),
            Some(Region::Matched { text: "foo", .. })
        ));

        assert!(matches!(
            regions.next(),
            Some(Region::Unmatched { text: " " })
        ));

        assert!(matches!(
            regions.next(),
            Some(Region::Matched { text: "bar", .. })
        ));
    }
}

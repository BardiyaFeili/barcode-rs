use std::error::Error;
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HighlightEvent};
use crossterm::style::Color;

pub struct SyntaxHighlighter {
    highlighter: Highlighter,
    configs: Vec<HighlightConfiguration>,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let mut configs = Vec::new();

        // Rust configuration
        let mut rust_config = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            "",
            "",
        ).unwrap();
        rust_config.configure(&HIGHLIGHT_NAMES);
        configs.push(rust_config);

        // Python configuration
        let mut python_config = HighlightConfiguration::new(
            tree_sitter_python::LANGUAGE.into(),
            "python",
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        ).unwrap();
        python_config.configure(&HIGHLIGHT_NAMES);
        configs.push(python_config);

        Self {
            highlighter: Highlighter::new(),
            configs,
        }
    }

    pub fn highlight(&mut self, content: &str, extension: &str) -> Result<Vec<(usize, usize, Color)>, Box<dyn Error>> {
        let config = match extension {
            "rs" => &self.configs[0],
            "py" => &self.configs[1],
            _ => return Ok(Vec::new()), // No highlighting for unknown extensions
        };

        let highlights = self.highlighter.highlight(
            config,
            content.as_bytes(),
            None,
            |_| None,
        )?;

        let mut result = Vec::new();
        let mut current_color = Color::Reset;

        for event in highlights {
            match event? {
                HighlightEvent::Source { start, end } => {
                    if current_color != Color::Reset {
                        result.push((start, end, current_color));
                    }
                }
                HighlightEvent::HighlightStart(s) => {
                    current_color = get_color_for_highlight(s.0);
                }
                HighlightEvent::HighlightEnd => {
                    current_color = Color::Reset;
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_highlight() {
        let mut highlighter = SyntaxHighlighter::new();
        let content = "fn main() { let x = 5; }";
        let highlights = highlighter.highlight(content, "rs").unwrap();
        
        // Check that we got some highlights
        assert!(!highlights.is_empty());
        
        // "fn" should be a keyword
        let fn_highlight = highlights.iter().find(|(s, e, _)| &content[*s..*e] == "fn");
        assert!(fn_highlight.is_some());
        assert_eq!(fn_highlight.unwrap().2, Color::Magenta); // keyword color
    }

    #[test]
    fn test_python_highlight() {
        let mut highlighter = SyntaxHighlighter::new();
        let content = "def main():\n    print('hello')";
        let highlights = highlighter.highlight(content, "py").unwrap();
        
        assert!(!highlights.is_empty());
        
        // "def" should be a keyword
        let def_highlight = highlights.iter().find(|(s, e, _)| &content[*s..*e] == "def");
        assert!(def_highlight.is_some());
        assert_eq!(def_highlight.unwrap().2, Color::Magenta); // keyword color
    }
}

const HIGHLIGHT_NAMES: [&str; 11] = [
    "keyword",
    "string",
    "comment",
    "function",
    "type",
    "variable",
    "constant",
    "operator",
    "attribute",
    "punctuation",
    "constructor",
];

fn get_color_for_highlight(idx: usize) -> Color {
    match HIGHLIGHT_NAMES.get(idx).copied() {
        Some("keyword") => Color::Magenta,
        Some("string") => Color::Green,
        Some("comment") => Color::Grey,
        Some("function") => Color::Blue,
        Some("type") => Color::Yellow,
        Some("variable") => Color::White,
        Some("constant") => Color::Red,
        Some("operator") => Color::Cyan,
        Some("attribute") => Color::DarkYellow,
        Some("punctuation") => Color::DarkGrey,
        Some("constructor") => Color::Blue,
        _ => Color::Reset,
    }
}

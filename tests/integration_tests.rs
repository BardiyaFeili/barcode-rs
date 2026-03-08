use barcode::highlight::SyntaxHighlighter;
use barcode::component::{Component, ComponentType};
use barcode::config::Config;
use crossterm::style::Color;
use std::time::Duration;

#[test]
fn test_rust_highlight_integration() {
    let mut highlighter = SyntaxHighlighter::new();
    let content = "fn main() { let x = 5; }";
    let highlights = highlighter.highlight(content, "rs").unwrap();
    
    assert!(!highlights.is_empty());
    
    let fn_highlight = highlights.iter().find(|(s, e, _)| &content[*s..*e] == "fn");
    assert!(fn_highlight.is_some());
    assert_eq!(fn_highlight.unwrap().2, Color::Magenta);
}

#[test]
fn test_python_highlight_integration() {
    let mut highlighter = SyntaxHighlighter::new();
    let content = "def main():\n    print('hello')";
    let highlights = highlighter.highlight(content, "py").unwrap();
    
    assert!(!highlights.is_empty());
    
    let def_highlight = highlights.iter().find(|(s, e, _)| &content[*s..*e] == "def");
    assert!(def_highlight.is_some());
    assert_eq!(def_highlight.unwrap().2, Color::Magenta);
}

#[test]
fn test_component_highlights_integration() {
    let config = Config::default();
    let mut highlighter = SyntaxHighlighter::new();
    let content = vec!["fn main() {".to_string(), "    let x = 5;".to_string(), "}".to_string()];
    let mut comp = Component::new(content, ComponentType::Buffer, Some("main.rs".to_string()), &config);
    
    // Simulate an update
    comp.update(Duration::from_millis(0), &config, &mut highlighter).unwrap();
    
    // Check that highlights were populated
    assert_eq!(comp.highlights.len(), 3);
    assert_eq!(comp.highlights[0][0], Some(Color::Magenta)); // 'f'
    assert_eq!(comp.highlights[0][1], Some(Color::Magenta)); // 'n'
}

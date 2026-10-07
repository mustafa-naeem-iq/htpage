use crate::ast::{ElementNode, Node, TextNode};
use crate::diagnostic::DiagnosticSeverity;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::span::Span;
use crate::validator::validate;

pub fn compile(source: &str) -> Result<String, String> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let mut nodes = parser.parse_program()?;

    // Semantic validation pass
    let diagnostics = validate(&nodes);
    if let Some(err) = diagnostics.iter().find(|d| d.severity == DiagnosticSeverity::Error) {
        return Err(err.message.clone());
    }

    // Collect scoped styles for pseudo-classes (hover, focus, etc.)
    let mut scoped_css = Vec::new();
    let mut scope_counter = 0;
    for node in &mut nodes {
        if let Node::Element(ref mut el) = node {
            el.collect_scoped_styles(&mut scoped_css, &mut scope_counter);
        }
    }

    let mut output = String::new();

    // If scoped styles exist, inject into <head> or prepend
    if !scoped_css.is_empty() {
        let style_content = format!(
            "\n{}\n",
            scoped_css
                .iter()
                .map(|r| format!("    {r}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let mut style_element = ElementNode::new("style".to_string(), Span::default(), Span::default());
        style_element
            .children
            .push(Node::Text(TextNode::new(style_content, Span::default())));

        let mut target_head: Option<&mut ElementNode> = None;
        for node in &mut nodes {
            if let Node::Element(ref mut el) = node {
                if el.name == "head" {
                    target_head = Some(el);
                    break;
                } else if el.name == "html" {
                    for child in &mut el.children {
                        if let Node::Element(ref mut cel) = child {
                            if cel.name == "head" {
                                target_head = Some(cel);
                                break;
                            }
                        }
                    }
                    if target_head.is_some() {
                        break;
                    }
                }
            }
        }

        if let Some(head) = target_head {
            head.children.push(Node::Element(style_element));
        } else {
            output.push_str(&style_element.to_html(0, false)?);
        }
    }

    for node in &mut nodes {
        if let Node::Element(ref mut el) = node {
            el.pre_process();
        }
        output.push_str(&node.to_html(0, false)?);
    }

    Ok(output)
}

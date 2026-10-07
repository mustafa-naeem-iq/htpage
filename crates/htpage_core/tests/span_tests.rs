use htpage_core::ast::Node;
use htpage_core::lexer::Lexer;
use htpage_core::parser::Parser;
use htpage_core::span::LineIndex;

#[test]
fn test_element_and_attribute_spans() {
    let source = "button { Click, disabled: true, class: btn }";
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let nodes = parser.parse_program().unwrap();

    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.name, "button");
            assert_eq!(el.name_span.start, 0);
            assert_eq!(el.name_span.end, 6);
            assert_eq!(el.span.start, 0);
            assert_eq!(el.span.end, 44);

            // Child text node
            assert_eq!(el.children.len(), 1);
            if let Node::Text(t) = &el.children[0] {
                assert_eq!(t.text, "Click");
                assert_eq!(t.span.start, 9);
                assert_eq!(t.span.end, 14);
            } else {
                panic!("Expected text child");
            }

            // Attributes
            assert_eq!(el.attrs.len(), 2);

            let attr0 = &el.attrs[0];
            assert_eq!(attr0.key, "disabled");
            assert_eq!(attr0.key_span.start, 16);
            assert_eq!(attr0.key_span.end, 24);
            assert_eq!(attr0.value, "true");
            assert_eq!(attr0.value_span.start, 26);
            assert_eq!(attr0.value_span.end, 30);

            let attr1 = &el.attrs[1];
            assert_eq!(attr1.key, "class");
            assert_eq!(attr1.key_span.start, 32);
            assert_eq!(attr1.key_span.end, 37);
            assert_eq!(attr1.value, "btn");
            assert_eq!(attr1.value_span.start, 39);
            assert_eq!(attr1.value_span.end, 42);
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_line_index() {
    let source = "line0\nline1\nline2 is longer\n";
    let index = LineIndex::new(source);

    assert_eq!(index.line_col(0), (0, 0));
    assert_eq!(index.line_col(3), (0, 3));
    assert_eq!(index.line_col(6), (1, 0));
    assert_eq!(index.line_col(8), (1, 2));
    assert_eq!(index.line_col(12), (2, 0));
    assert_eq!(index.line_col(18), (2, 6));
}

#[test]
fn test_raw_css_block_span() {
    let source = "css { body { color: red; } }";
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let nodes = parser.parse_program().unwrap();

    assert_eq!(nodes.len(), 1);
    if let Node::Element(el) = &nodes[0] {
        assert_eq!(el.name, "css");
        assert_eq!(el.name_span.start, 0);
        assert_eq!(el.name_span.end, 3);
        assert_eq!(el.span.start, 0);
        assert_eq!(el.span.end, 28);
    } else {
        panic!("Expected css element");
    }
}

#[test]
fn test_validator_diagnostic_span() {
    let source = "img { href: invalid_attr }";
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let nodes = parser.parse_program().unwrap();

    let diags = htpage_core::validate(&nodes);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "Attribute 'href' is not valid for tag '<img>'!");
    // "href" is at offset 6..10
    assert_eq!(diags[0].span.start, 6);
    assert_eq!(diags[0].span.end, 10);
}

#[test]
fn test_parse_with_diagnostics_integration() {
    let source = "div { Welcome, class: card }\nimg { href: bad }\n";
    let output = htpage_core::parse_with_diagnostics(source);

    // Both nodes parsed
    assert_eq!(output.ast.len(), 2);
    // Diagnostic captured for img href
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(output.diagnostics[0].message, "Attribute 'href' is not valid for tag '<img>'!");
    // LineIndex works
    assert_eq!(output.line_index.line_col(output.diagnostics[0].span.start), (1, 6));
}



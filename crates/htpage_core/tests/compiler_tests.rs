use htpage_core::compile;

#[test]
fn test_basic_page() {
    let input = r#"
doctype: html
html {
    head {
        title: Hello
    }
    body {
        h1 { Welcome, color: red }
    }
}
"#;
    let expected = "<!DOCTYPE html>\n<html>\n    <head>\n        <title>Hello</title>\n    </head>\n    <body>\n        <h1 style=\"color: red\">Welcome</h1>\n    </body>\n</html>\n";
    assert_eq!(compile(input).unwrap(), expected);
}

#[test]
fn test_boolean_attributes() {
    let input = "button { Click, disabled: true }\nbutton { Active, disabled: false }\n";
    let output = compile(input).unwrap();
    assert!(output.contains("<button disabled>Click</button>"));
    assert!(output.contains("<button>Active</button>"));
}

#[test]
fn test_data_and_aria_attributes() {
    let input = "div { Content, data-test: sample, aria-hidden: true }\n";
    let output = compile(input).unwrap();
    assert_eq!(output, "<div aria-hidden=\"true\" data-test=\"sample\">Content</div>\n");
}

#[test]
fn test_character_escaping() {
    let input = "p { literal \\{brace\\} and \\:colon and \\#hash }\n";
    let output = compile(input).unwrap();
    assert_eq!(output, "<p>literal {brace} and :colon and #hash</p>\n");
}

#[test]
fn test_void_tags_bracket_and_brace() {
    let input = "p { line 1 {br} line 2 [hr] line 3 }\n";
    let output = compile(input).unwrap();
    assert!(output.contains("<br>"));
    assert!(output.contains("<hr>"));
}

#[test]
fn test_pseudo_classes_and_scoped_style() {
    let input = "p { Button, color: red, hover:color: blue }\n";
    let output = compile(input).unwrap();
    assert!(output.contains("<style>"));
    assert!(output.contains(".ht-1:hover { color: blue; }"));
    assert!(output.contains("<p class=\"ht-1\" style=\"color: red\">Button</p>"));
}

#[test]
fn test_html_only_attribute_validation() {
    let input = "img { href: invalid_attr }\n";
    let result = compile(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Attribute 'href' is not valid for tag '<img>'!"));
}

#[test]
fn test_nested_block_comma_lookahead() {
    let input = "button { Click, hover { color: blue, font-weight: 600 } }\n";
    let output = compile(input).unwrap();
    assert!(output.contains("<style>"));
    assert!(output.contains(".ht-1:hover { color: blue; font-weight: 600; }"));
    assert!(output.contains("<button class=\"ht-1\">Click</button>"));
    assert!(!output.contains("Click,"));
}


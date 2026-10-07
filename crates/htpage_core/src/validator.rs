use crate::ast::{get_tag_attrs, Node, GLOBAL_ATTRS, HTML_ONLY_ATTRS};
use crate::diagnostic::Diagnostic;

pub fn validate(nodes: &[Node]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for node in nodes {
        validate_node(node, &mut diagnostics);
    }
    diagnostics
}

fn validate_node(node: &Node, diagnostics: &mut Vec<Diagnostic>) {
    match node {
        Node::Element(el) => {
            let tag_name = match el.name.as_str() {
                "css" => "style",
                "javascript" => "script",
                _ => el.name.as_str(),
            };

            let mut allowed_tag_attrs = get_tag_attrs(&el.name);
            if allowed_tag_attrs.is_empty() {
                allowed_tag_attrs = get_tag_attrs(tag_name);
            }

            for attr in &el.attrs {
                let k = &attr.key;
                let is_html_only = HTML_ONLY_ATTRS.contains(&k.as_str());
                if is_html_only {
                    let is_allowed = GLOBAL_ATTRS.contains(&k.as_str())
                        || allowed_tag_attrs.contains(&k.as_str());
                    if !is_allowed {
                        diagnostics.push(Diagnostic::error(
                            attr.key_span,
                            format!("Attribute '{k}' is not valid for tag '<{}>'!", el.name),
                        ));
                    }
                }
            }

            for child in &el.children {
                validate_node(child, diagnostics);
            }
        }
        Node::Text(_) => {}
    }
}

use std::collections::{BTreeMap, HashMap};

pub const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr",
];

pub const GLOBAL_ATTRS: &[&str] = &[
    "id", "class", "title", "lang", "dir", "tabindex", "hidden", "for",
];

pub const INLINE_TAGS: &[&str] = &[
    "b", "i", "u", "em", "strong", "span", "a", "small", "code", "sub", "sup", "br", "wbr",
];

pub const BOOLEAN_ATTRS: &[&str] = &[
    "disabled", "required", "checked", "readonly", "multiple", "autofocus", "autoplay", "controls",
    "loop", "muted", "defer", "async", "hidden", "open", "ismap", "reversed", "novalidate",
    "formnovalidate",
];

pub const HTML_ONLY_ATTRS: &[&str] = &[
    "href", "src", "alt", "rel", "target", "name", "value", "charset", "content", "placeholder",
    "disabled", "required", "checked", "action", "method", "enctype", "allow", "loading", "download",
    "property", "media", "for",
];

fn get_tag_attrs(tag: &str) -> &'static [&'static str] {
    match tag {
        "a" => &["href", "target", "rel", "type", "download"],
        "img" => &["src", "alt", "width", "height", "loading"],
        "meta" => &["name", "content", "charset", "property"],
        "script" | "javascript" => &["src", "type", "async", "defer"],
        "link" => &["rel", "href", "type", "media"],
        "input" => &[
            "type", "name", "value", "placeholder", "disabled", "required", "checked", "id",
            "readonly", "autofocus",
        ],
        "form" => &["action", "method", "enctype", "novalidate"],
        "button" => &["type", "disabled", "name", "value", "autofocus"],
        "label" => &["for", "id", "form"],
        "iframe" => &["src", "width", "height", "allow", "allowfullscreen", "loading"],
        _ => &[],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Text(TextNode),
    Element(ElementNode),
}

impl Node {
    pub fn to_html(&self, indent: usize, inline: bool) -> Result<String, String> {
        match self {
            Node::Text(t) => Ok(t.to_html(indent, inline)),
            Node::Element(el) => el.to_html(indent, inline),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextNode {
    pub text: String,
}

impl TextNode {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn to_html(&self, indent: usize, inline: bool) -> String {
        if inline {
            self.text.clone()
        } else {
            format!("{}{}\n", "    ".repeat(indent), self.text)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementNode {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
    pub pseudo_rules: HashMap<String, Vec<(String, String)>>,
}

impl ElementNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            attrs: Vec::new(),
            children: Vec::new(),
            pseudo_rules: HashMap::new(),
        }
    }

    pub fn default_attr(&self) -> Option<&'static str> {
        match self.name.as_str() {
            "img" => Some("src"),
            "a" | "link" => Some("href"),
            "meta" => Some("charset"),
            "script" | "javascript" => Some("src"),
            _ => None,
        }
    }

    pub fn pre_process(&mut self) {
        if VOID_TAGS.contains(&self.name.as_str()) {
            for child in &self.children {
                if let Node::Text(t) = child {
                    if let Some(attr_name) = self.default_attr() {
                        if !self.attrs.iter().any(|(k, _)| k == attr_name) {
                            self.attrs.push((attr_name.to_string(), t.text.clone()));
                        }
                    }
                }
            }
            self.children.clear();
        }

        for child in &mut self.children {
            if let Node::Element(ref mut el) = child {
                el.pre_process();
            }
        }
    }

    pub fn all_inline_children(&self) -> bool {
        if self.children.is_empty() {
            return false;
        }
        self.children.iter().all(|c| match c {
            Node::Text(_) => true,
            Node::Element(el) => INLINE_TAGS.contains(&el.name.as_str()),
        })
    }

    pub fn collect_scoped_styles(&mut self, collector: &mut Vec<String>, counter: &mut usize) {
        if !self.pseudo_rules.is_empty() {
            *counter += 1;
            let cls = format!("ht-{}", counter);

            if let Some((_, existing_class)) = self.attrs.iter_mut().find(|(k, _)| k == "class") {
                existing_class.push(' ');
                existing_class.push_str(&cls);
            } else {
                self.attrs.push(("class".to_string(), cls.clone()));
            }

            // Sort pseudo-class keys for deterministic output
            let mut sorted_keys: Vec<_> = self.pseudo_rules.keys().collect();
            sorted_keys.sort();

            for p_name in sorted_keys {
                let rules = &self.pseudo_rules[p_name];
                let decls = rules
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join("; ");
                collector.push(format!(".{cls}:{p_name} {{ {decls}; }}"));
            }
        }

        for child in &mut self.children {
            if let Node::Element(ref mut el) = child {
                el.collect_scoped_styles(collector, counter);
            }
        }
    }

    pub fn to_html(&self, indent: usize, inline: bool) -> Result<String, String> {
        if self.name == "doctype" {
            let mut val = "html".to_string();
            if let Some(Node::Text(t)) = self.children.first() {
                val = t.text.clone();
            }
            return Ok(format!("<!DOCTYPE {val}>\n"));
        }

        let ind = if inline {
            String::new()
        } else {
            "    ".repeat(indent)
        };

        let tag_name = match self.name.as_str() {
            "css" => "style",
            "javascript" => "script",
            _ => self.name.as_str(),
        };

        let mut b = format!("{ind}<{tag_name}");

        let mut html_attrs: BTreeMap<String, Option<String>> = BTreeMap::new();
        let mut css_props: Vec<String> = Vec::new();

        let allowed_tag_attrs = {
            let mut attrs = get_tag_attrs(self.name.as_str());
            if attrs.is_empty() {
                attrs = get_tag_attrs(tag_name);
            }
            attrs
        };

        for (k, v) in &self.attrs {
            let is_html = GLOBAL_ATTRS.contains(&k.as_str())
                || allowed_tag_attrs.contains(&k.as_str())
                || k.starts_with("on")
                || k.starts_with("data-")
                || k.starts_with("aria-");

            if is_html {
                if BOOLEAN_ATTRS.contains(&k.as_str()) {
                    if v == "true" {
                        html_attrs.insert(k.clone(), None);
                    } else if v == "false" {
                        // Omit when false
                    } else {
                        html_attrs.insert(k.clone(), Some(v.clone()));
                    }
                } else {
                    html_attrs.insert(k.clone(), Some(v.clone()));
                }
            } else if HTML_ONLY_ATTRS.contains(&k.as_str()) {
                return Err(format!(
                    "Attribute '{k}' is not valid for tag '<{}>'!",
                    self.name
                ));
            } else {
                css_props.push(format!("{k}: {v}"));
            }
        }

        if !css_props.is_empty() {
            html_attrs.insert("style".to_string(), Some(css_props.join("; ")));
        }

        for (k, v_opt) in html_attrs {
            match v_opt {
                None => b.push_str(&format!(" {k}")),
                Some(v) => b.push_str(&format!(" {k}=\"{v}\"")),
            }
        }

        if VOID_TAGS.contains(&tag_name) {
            b.push_str(if inline { ">" } else { ">\n" });
            return Ok(b);
        }

        b.push('>');

        if tag_name == "style" || tag_name == "script" {
            let raw: String = self
                .children
                .iter()
                .map(|c| c.to_html(0, true))
                .collect::<Result<Vec<_>, _>>()?
                .join("");
            if !raw.trim().is_empty() {
                b.push_str(&raw);
            }
            b.push_str(&format!("</{tag_name}>\n"));
            return Ok(b);
        }

        // Format inline elements seamlessly if all children are inline
        let block_level = ["html", "head", "body", "ul", "ol", "table", "form"];
        if self.all_inline_children() && !block_level.contains(&tag_name) {
            for (idx, c) in self.children.iter().enumerate() {
                let res = c.to_html(0, true)?;
                if idx > 0 {
                    let prev = &self.children[idx - 1];
                    match (prev, c) {
                        (Node::Element(_), Node::Text(_)) => {
                            if !res.starts_with([',', '.', '!', '?', ';', ':']) {
                                b.push(' ');
                            }
                        }
                        (Node::Text(t), Node::Element(_)) => {
                            if !t.text.ends_with(' ') {
                                b.push(' ');
                            }
                        }
                        _ => {}
                    }
                }
                b.push_str(&res);
            }
            b.push_str(&format!("</{tag_name}>"));
            if !inline {
                b.push('\n');
            }
            return Ok(b);
        }

        if self.children.len() == 1 {
            if let Node::Text(t) = &self.children[0] {
                b.push_str(&t.to_html(0, true));
                b.push_str(&format!("</{tag_name}>"));
                if !inline {
                    b.push('\n');
                }
                return Ok(b);
            }
        }

        if !self.children.is_empty() {
            b.push('\n');
            for c in &self.children {
                b.push_str(&c.to_html(indent + 1, false)?);
            }
            b.push_str(&ind);
        }

        b.push_str(&format!("</{tag_name}>\n"));
        Ok(b)
    }
}

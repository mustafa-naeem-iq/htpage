use crate::ast::{ElementNode, Node, TextNode, GLOBAL_ATTRS, VOID_TAGS};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

pub const KNOWN_HTML_TAGS: &[&str] = &[
    "html", "head", "body", "title", "meta", "link", "style", "script", "css", "javascript",
    "h1", "h2", "h3", "h4", "h5", "h6", "p", "a", "img", "br", "hr", "div", "span", "b", "i",
    "u", "em", "strong", "ul", "ol", "li", "table", "tr", "td", "th", "form", "input", "button",
    "label", "select", "option", "textarea", "nav", "header", "footer", "main", "section",
    "article", "aside", "figure", "figcaption", "audio", "video", "source", "canvas", "svg",
    "iframe", "doctype",
];

pub const PSEUDO_CLASSES: &[&str] = &[
    "hover",
    "focus",
    "active",
    "visited",
    "before",
    "after",
    "first-child",
    "last-child",
    "nth-child",
    "disabled",
    "checked",
];

pub struct Parser {
    lexer: Lexer,
    curr: Option<Token>,
    peek: Option<Token>,
    peek2: Option<Token>,
}

pub struct MixedResult {
    pub text_before: Option<TextNode>,
    pub node: Option<Node>,
    pub attr_name: Option<String>,
    pub attr_val: Option<String>,
    pub is_attr: bool,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let t1 = lexer.next_token();
        let t2 = lexer.next_token();
        let t3 = lexer.next_token();
        Self {
            lexer,
            curr: Some(t1),
            peek: Some(t2),
            peek2: Some(t3),
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        while !self.is_at_end() {
            if self.match_sep() || self.match_token(|t| matches!(t, TokenType::Comma)) {
                continue;
            }

            let res = self.parse_mixed(None)?;
            let had_content = res.text_before.is_some() || res.node.is_some() || res.is_attr;
            if let Some(tb) = res.text_before {
                nodes.push(Node::Text(tb));
            }
            if !res.is_attr {
                if let Some(n) = res.node {
                    nodes.push(n);
                }
            }
            if !had_content {
                self.advance();
            }
        }
        Ok(nodes)
    }

    fn advance(&mut self) -> Option<Token> {
        let old = self.curr.take();
        self.curr = self.peek.take();
        self.peek = self.peek2.take();
        self.peek2 = Some(self.lexer.next_token());
        old
    }

    fn match_token<F>(&mut self, predicate: F) -> bool
    where
        F: FnOnce(&TokenType) -> bool,
    {
        if let Some(curr) = &self.curr {
            if predicate(&curr.token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn match_sep(&mut self) -> bool {
        self.match_token(|t| matches!(t, TokenType::Sep(_)))
    }

    fn check_token<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(&TokenType) -> bool,
    {
        if let Some(curr) = &self.curr {
            predicate(&curr.token_type)
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        match &self.curr {
            None => true,
            Some(t) => matches!(t.token_type, TokenType::Eof),
        }
    }

    fn is_attr_comma(&self) -> bool {
        if !self.check_token(|t| matches!(t, TokenType::Comma)) {
            return false;
        }
        let peek_is_word = matches!(&self.peek, Some(t) if matches!(t.token_type, TokenType::Word(_)));
        let peek2_is_colon = matches!(&self.peek2, Some(t) if matches!(t.token_type, TokenType::Colon));
        peek_is_word && peek2_is_colon
    }

    fn parse_mixed(&mut self, parent_tag: Option<&mut ElementNode>) -> Result<MixedResult, String> {
        // 1. Void tag in bracket form: [br] or [hr]
        if self.match_token(|t| matches!(t, TokenType::LBracket)) {
            let mut words = Vec::new();
            while self.check_token(|t| matches!(t, TokenType::Word(_))) {
                if let Some(Token {
                    token_type: TokenType::Word(w),
                    ..
                }) = self.advance()
                {
                    words.push(w);
                }
            }
            self.match_token(|t| matches!(t, TokenType::RBracket));
            return Ok(MixedResult {
                text_before: None,
                node: Some(Node::Element(ElementNode::new(words.join(" ")))),
                attr_name: None,
                attr_val: None,
                is_attr: false,
            });
        }

        // 2. Void tag in brace form: {br} or {hr}
        let is_brace_void = if self.check_token(|t| matches!(t, TokenType::LBrace)) {
            if let Some(peek) = &self.peek {
                if let TokenType::Word(w) = &peek.token_type {
                    if VOID_TAGS.contains(&w.as_str()) {
                        matches!(&self.peek2, Some(p2) if matches!(p2.token_type, TokenType::RBrace))
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if is_brace_void {
            self.match_token(|t| matches!(t, TokenType::LBrace));
            let void_name = if let Some(Token {
                token_type: TokenType::Word(w),
                ..
            }) = self.advance()
            {
                w
            } else {
                String::new()
            };
            self.match_token(|t| matches!(t, TokenType::RBrace));
            return Ok(MixedResult {
                text_before: None,
                node: Some(Node::Element(ElementNode::new(void_name))),
                attr_name: None,
                attr_val: None,
                is_attr: false,
            });
        }

        let mut words: Vec<String> = Vec::new();
        while self.check_token(|t| matches!(t, TokenType::Word(_)))
            || (self.check_token(|t| matches!(t, TokenType::Comma)) && !self.is_attr_comma())
        {
            if self.match_token(|t| matches!(t, TokenType::Comma)) {
                if let Some(last) = words.last_mut() {
                    last.push(',');
                } else {
                    words.push(",".to_string());
                }
            } else if let Some(Token {
                token_type: TokenType::Word(w),
                ..
            }) = self.advance()
            {
                words.push(w);
            }
        }

        // 3. Check for inline void element immediately following words: e.g. text {br}
        let is_inline_brace_void = if self.check_token(|t| matches!(t, TokenType::LBrace)) {
            if let Some(peek) = &self.peek {
                if let TokenType::Word(w) = &peek.token_type {
                    if VOID_TAGS.contains(&w.as_str()) {
                        matches!(&self.peek2, Some(p2) if matches!(p2.token_type, TokenType::RBrace))
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if is_inline_brace_void {
            self.match_token(|t| matches!(t, TokenType::LBrace));
            let void_name = if let Some(Token {
                token_type: TokenType::Word(w),
                ..
            }) = self.advance()
            {
                w
            } else {
                String::new()
            };
            self.match_token(|t| matches!(t, TokenType::RBrace));
            let text_before = if !words.is_empty() {
                Some(TextNode::new(words.join(" ")))
            } else {
                None
            };
            return Ok(MixedResult {
                text_before,
                node: Some(Node::Element(ElementNode::new(void_name))),
                attr_name: None,
                attr_val: None,
                is_attr: false,
            });
        }

        if self.match_token(|t| matches!(t, TokenType::Colon)) {
            let name = words.pop().unwrap_or_default();

            // Support pseudo-class prefix property shorthand: hover:color: blue
            let is_pseudo_prefix = PSEUDO_CLASSES.contains(&name.as_str())
                && self.check_token(|t| matches!(t, TokenType::Word(_)))
                && matches!(&self.peek, Some(t) if matches!(t.token_type, TokenType::Colon));

            if let Some(parent) = parent_tag {
                if is_pseudo_prefix {
                    let prop_name = if let Some(Token {
                        token_type: TokenType::Word(w),
                        ..
                    }) = self.advance()
                    {
                        w
                    } else {
                        String::new()
                    };
                    self.match_token(|t| matches!(t, TokenType::Colon));
                    let prop_val = self.parse_attribute_value();

                    let rules = parent.pseudo_rules.entry(name).or_default();
                    if let Some(existing) = rules.iter_mut().find(|(k, _)| k == &prop_name) {
                        existing.1 = prop_val;
                    } else {
                        rules.push((prop_name, prop_val));
                    }

                    let text_before = if !words.is_empty() {
                        Some(TextNode::new(words.join(" ")))
                    } else {
                        None
                    };
                    return Ok(MixedResult {
                        text_before,
                        node: None,
                        attr_name: None,
                        attr_val: None,
                        is_attr: false,
                    });
                }

                let val = self.parse_attribute_value();
                let mut is_tag_shortcut = false;
                if KNOWN_HTML_TAGS.contains(&name.as_str()) {
                    if GLOBAL_ATTRS.contains(&name.as_str()) {
                        if parent.name == "head" || parent.name == "html" {
                            is_tag_shortcut = true;
                        }
                    } else {
                        is_tag_shortcut = true;
                    }
                }

                let text_before = if !words.is_empty() {
                    Some(TextNode::new(words.join(" ")))
                } else {
                    None
                };

                if is_tag_shortcut {
                    let mut node = ElementNode::new(name);
                    node.children.push(Node::Text(TextNode::new(val)));
                    return Ok(MixedResult {
                        text_before,
                        node: Some(Node::Element(node)),
                        attr_name: None,
                        attr_val: None,
                        is_attr: false,
                    });
                } else {
                    return Ok(MixedResult {
                        text_before,
                        node: None,
                        attr_name: Some(name),
                        attr_val: Some(val),
                        is_attr: true,
                    });
                }
            } else {
                let val = self.parse_attribute_value();
                let mut is_tag_shortcut = false;
                if KNOWN_HTML_TAGS.contains(&name.as_str()) {
                    is_tag_shortcut = true;
                }

                let text_before = if !words.is_empty() {
                    Some(TextNode::new(words.join(" ")))
                } else {
                    None
                };

                if is_tag_shortcut {
                    let mut node = ElementNode::new(name);
                    node.children.push(Node::Text(TextNode::new(val)));
                    return Ok(MixedResult {
                        text_before,
                        node: Some(Node::Element(node)),
                        attr_name: None,
                        attr_val: None,
                        is_attr: false,
                    });
                } else {
                    return Ok(MixedResult {
                        text_before,
                        node: None,
                        attr_name: Some(name),
                        attr_val: Some(val),
                        is_attr: true,
                    });
                }
            }
        } else if self.check_token(|t| matches!(t, TokenType::LBrace)) {
            let name = words.pop().unwrap_or_default();

            // Nested pseudo-class block: hover { color: blue }
            if PSEUDO_CLASSES.contains(&name.as_str()) && parent_tag.is_some() {
                let mut pseudo_node = ElementNode::new(name.clone());
                self.match_token(|t| matches!(t, TokenType::LBrace));
                self.parse_block(&mut pseudo_node)?;

                let parent = parent_tag.unwrap();
                let rules = parent.pseudo_rules.entry(name).or_default();
                for (k, v) in pseudo_node.attrs {
                    if let Some(existing) = rules.iter_mut().find(|(ek, _)| ek == &k) {
                        existing.1 = v;
                    } else {
                        rules.push((k, v));
                    }
                }

                let text_before = if !words.is_empty() {
                    Some(TextNode::new(words.join(" ")))
                } else {
                    None
                };
                return Ok(MixedResult {
                    text_before,
                    node: None,
                    attr_name: None,
                    attr_val: None,
                    is_attr: false,
                });
            }

            let brace_pos = self.curr.as_ref().map(|t| t.pos).unwrap_or(0);
            let mut tag = ElementNode::new(name);
            self.match_token(|t| matches!(t, TokenType::LBrace));

            if ["script", "style", "css", "javascript"].contains(&tag.name.as_str()) {
                let val = self.lexer.read_raw_until_matching_brace(brace_pos + 1);
                self.curr = Some(self.lexer.next_token());
                self.peek = Some(self.lexer.next_token());
                self.peek2 = Some(self.lexer.next_token());
                tag.children.push(Node::Text(TextNode::new(val)));
            } else {
                self.parse_block(&mut tag)?;
            }

            let text_before = if !words.is_empty() {
                Some(TextNode::new(words.join(" ")))
            } else {
                None
            };
            return Ok(MixedResult {
                text_before,
                node: Some(Node::Element(tag)),
                attr_name: None,
                attr_val: None,
                is_attr: false,
            });
        }

        if !words.is_empty() {
            return Ok(MixedResult {
                text_before: None,
                node: Some(Node::Text(TextNode::new(words.join(" ")))),
                attr_name: None,
                attr_val: None,
                is_attr: false,
            });
        }

        Ok(MixedResult {
            text_before: None,
            node: None,
            attr_name: None,
            attr_val: None,
            is_attr: false,
        })
    }

    fn parse_block(&mut self, tag: &mut ElementNode) -> Result<(), String> {
        while !self.check_token(|t| matches!(t, TokenType::RBrace)) && !self.is_at_end() {
            if self.match_sep()
                || (self.match_token(|t| matches!(t, TokenType::Comma)) && !self.is_attr_comma())
            {
                continue;
            }

            let res = self.parse_mixed(Some(tag))?;
            let had_content = res.text_before.is_some() || res.node.is_some() || res.is_attr;

            if let Some(tb) = res.text_before {
                tag.children.push(Node::Text(tb));
            }

            if res.is_attr {
                if let (Some(k), Some(v)) = (res.attr_name, res.attr_val) {
                    tag.attrs.push((k, v));
                }
            } else if let Some(n) = res.node {
                tag.children.push(n);
            }

            if !had_content {
                self.advance();
            }
        }
        self.match_token(|t| matches!(t, TokenType::RBrace));
        Ok(())
    }

    fn parse_attribute_value(&mut self) -> String {
        let is_paren = self.check_token(|t| matches!(t, TokenType::LParen));
        let is_brace = self.check_token(|t| matches!(t, TokenType::LBrace));

        if is_paren || is_brace {
            let is_paren_mode = is_paren;
            if is_paren {
                self.match_token(|t| matches!(t, TokenType::LParen));
            } else {
                self.match_token(|t| matches!(t, TokenType::LBrace));
            }

            let mut res = String::new();
            let mut depth = 1;

            while !self.is_at_end() {
                if let Some(tok) = &self.curr {
                    let is_matching_closing = if is_paren_mode {
                        matches!(tok.token_type, TokenType::RParen)
                    } else {
                        matches!(tok.token_type, TokenType::RBrace)
                    };

                    let is_matching_opening = if is_paren_mode {
                        matches!(tok.token_type, TokenType::LParen)
                    } else {
                        matches!(tok.token_type, TokenType::LBrace)
                    };

                    if is_matching_closing {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    } else if is_matching_opening {
                        depth += 1;
                    }
                }

                if let Some(tok) = self.advance() {
                    match tok.token_type {
                        TokenType::Word(w) => {
                            res.push_str(&w);
                            if self.check_token(|t| matches!(t, TokenType::Word(_))) {
                                res.push(' ');
                            }
                        }
                        TokenType::Colon => res.push(':'),
                        TokenType::Comma => res.push_str(", "),
                        TokenType::Assign => res.push('='),
                        TokenType::Sep(_) => res.push(' '),
                        TokenType::LParen => res.push('('),
                        TokenType::RParen => res.push(')'),
                        TokenType::LBrace => res.push('{'),
                        TokenType::RBrace => res.push('}'),
                        TokenType::LBracket => res.push('['),
                        TokenType::RBracket => res.push(']'),
                        TokenType::Eof => break,
                    }
                }
            }

            if is_paren_mode {
                self.match_token(|t| matches!(t, TokenType::RParen));
            } else {
                self.match_token(|t| matches!(t, TokenType::RBrace));
            }

            while res.contains("  ") {
                res = res.replace("  ", " ");
            }
            return res.trim().to_string();
        }

        let mut words = Vec::new();
        while self.check_token(|t| matches!(t, TokenType::Word(_))) {
            if let Some(Token {
                token_type: TokenType::Word(w),
                ..
            }) = &self.curr
            {
                if PSEUDO_CLASSES.contains(&w.as_str())
                    && matches!(&self.peek, Some(t) if matches!(t.token_type, TokenType::LBrace))
                {
                    break;
                }
            }

            if let Some(Token {
                token_type: TokenType::Word(w),
                ..
            }) = self.advance()
            {
                words.push(w);
            }
        }
        words.join(" ")
    }
}

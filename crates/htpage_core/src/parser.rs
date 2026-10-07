use crate::ast::{Attribute, ElementNode, Node, TextNode, GLOBAL_ATTRS, VOID_TAGS};
use crate::lexer::Lexer;
use crate::span::Span;
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
    pub attr: Option<Attribute>,
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
            if self.match_sep() || self.match_token(|t| matches!(t, TokenType::Comma)).is_some() {
                continue;
            }

            let res = self.parse_mixed(None)?;
            let had_content = res.text_before.is_some() || res.node.is_some() || res.attr.is_some();
            if let Some(tb) = res.text_before {
                nodes.push(Node::Text(tb));
            }
            if let Some(n) = res.node {
                nodes.push(n);
            }
            if !had_content {
                self.advance();
            }
        }
        Ok(nodes)
    }

    pub fn parse_program_tolerant(&mut self) -> (Vec<Node>, Vec<crate::diagnostic::Diagnostic>) {
        let mut nodes = Vec::new();
        let mut diagnostics = Vec::new();
        while !self.is_at_end() {
            if self.match_sep() || self.match_token(|t| matches!(t, TokenType::Comma)).is_some() {
                continue;
            }

            match self.parse_mixed(None) {
                Ok(res) => {
                    let had_content = res.text_before.is_some() || res.node.is_some() || res.attr.is_some();
                    if let Some(tb) = res.text_before {
                        nodes.push(Node::Text(tb));
                    }
                    if let Some(n) = res.node {
                        nodes.push(n);
                    }
                    if !had_content {
                        self.advance();
                    }
                }
                Err(err) => {
                    let span = self.curr.as_ref().map(|t| t.span).unwrap_or_default();
                    diagnostics.push(crate::diagnostic::Diagnostic::error(span, err));
                    while !self.is_at_end()
                        && !self.check_token(|t| matches!(t, TokenType::Sep(_) | TokenType::Comma | TokenType::RBrace))
                    {
                        self.advance();
                    }
                }
            }
        }
        (nodes, diagnostics)
    }

    fn advance(&mut self) -> Option<Token> {
        let old = self.curr.take();
        self.curr = self.peek.take();
        self.peek = self.peek2.take();
        self.peek2 = Some(self.lexer.next_token());
        old
    }

    fn match_token<F>(&mut self, predicate: F) -> Option<Token>
    where
        F: FnOnce(&TokenType) -> bool,
    {
        if let Some(curr) = &self.curr {
            if predicate(&curr.token_type) {
                return self.advance();
            }
        }
        None
    }

    fn match_sep(&mut self) -> bool {
        self.match_token(|t| matches!(t, TokenType::Sep(_))).is_some()
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
        let peek2_is_delimiter = matches!(&self.peek2, Some(t) if matches!(t.token_type, TokenType::Colon | TokenType::LBrace));
        peek_is_word && peek2_is_delimiter
    }

    fn parse_mixed(&mut self, parent_tag: Option<&mut ElementNode>) -> Result<MixedResult, String> {
        // 1. Void tag in bracket form: [br] or [hr]
        if let Some(lbracket) = self.match_token(|t| matches!(t, TokenType::LBracket)) {
            let start = lbracket.span.start;
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
            let end = self
                .match_token(|t| matches!(t, TokenType::RBracket))
                .map(|t| t.span.end)
                .unwrap_or(start);
            let name = words.join(" ");
            let span = Span::new(start, end);
            return Ok(MixedResult {
                text_before: None,
                node: Some(Node::Element(ElementNode::new(name, span, span))),
                attr: None,
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
            let lbrace = self.match_token(|t| matches!(t, TokenType::LBrace)).unwrap();
            let start = lbrace.span.start;
            let void_name = if let Some(Token {
                token_type: TokenType::Word(w),
                ..
            }) = self.advance()
            {
                w
            } else {
                String::new()
            };
            let end = self
                .match_token(|t| matches!(t, TokenType::RBrace))
                .map(|t| t.span.end)
                .unwrap_or(start);
            let span = Span::new(start, end);
            return Ok(MixedResult {
                text_before: None,
                node: Some(Node::Element(ElementNode::new(void_name, span, span))),
                attr: None,
            });
        }

        let mut words: Vec<(String, Span)> = Vec::new();
        while self.check_token(|t| matches!(t, TokenType::Word(_)))
            || (self.check_token(|t| matches!(t, TokenType::Comma)) && !self.is_attr_comma())
        {
            if let Some(comma_tok) = self.match_token(|t| matches!(t, TokenType::Comma)) {
                if let Some(last) = words.last_mut() {
                    last.0.push(',');
                    last.1.end = comma_tok.span.end;
                } else {
                    words.push((",".to_string(), comma_tok.span));
                }
            } else if let Some(Token {
                token_type: TokenType::Word(w),
                span,
            }) = self.advance()
            {
                words.push((w, span));
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
            let lbrace = self.match_token(|t| matches!(t, TokenType::LBrace)).unwrap();
            let start = lbrace.span.start;
            let void_name = if let Some(Token {
                token_type: TokenType::Word(w),
                ..
            }) = self.advance()
            {
                w
            } else {
                String::new()
            };
            let end = self
                .match_token(|t| matches!(t, TokenType::RBrace))
                .map(|t| t.span.end)
                .unwrap_or(start);
            let text_before = if !words.is_empty() {
                let text = words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join(" ");
                let text_span = Span::new(words[0].1.start, words.last().unwrap().1.end);
                Some(TextNode::new(text, text_span))
            } else {
                None
            };
            let void_span = Span::new(start, end);
            return Ok(MixedResult {
                text_before,
                node: Some(Node::Element(ElementNode::new(void_name, void_span, void_span))),
                attr: None,
            });
        }

        if self.match_token(|t| matches!(t, TokenType::Colon)).is_some() {
            let (name, name_span) = words.pop().unwrap_or_else(|| (String::new(), Span::default()));

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
                    let (prop_val, _) = self.parse_attribute_value();

                    let rules = parent.pseudo_rules.entry(name).or_default();
                    if let Some(existing) = rules.iter_mut().find(|(k, _)| k == &prop_name) {
                        existing.1 = prop_val;
                    } else {
                        rules.push((prop_name, prop_val));
                    }

                    let text_before = if !words.is_empty() {
                        let text = words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join(" ");
                        let text_span = Span::new(words[0].1.start, words.last().unwrap().1.end);
                        Some(TextNode::new(text, text_span))
                    } else {
                        None
                    };
                    return Ok(MixedResult {
                        text_before,
                        node: None,
                        attr: None,
                    });
                }

                let (val, val_span) = self.parse_attribute_value();
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
                    let text = words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join(" ");
                    let text_span = Span::new(words[0].1.start, words.last().unwrap().1.end);
                    Some(TextNode::new(text, text_span))
                } else {
                    None
                };

                if is_tag_shortcut {
                    let element_span = Span::new(name_span.start, val_span.end);
                    let mut node = ElementNode::new(name, name_span, element_span);
                    node.children.push(Node::Text(TextNode::new(val, val_span)));
                    return Ok(MixedResult {
                        text_before,
                        node: Some(Node::Element(node)),
                        attr: None,
                    });
                } else {
                    let attr = Attribute::new(name, name_span, val, val_span);
                    return Ok(MixedResult {
                        text_before,
                        node: None,
                        attr: Some(attr),
                    });
                }
            } else {
                let (val, val_span) = self.parse_attribute_value();
                let mut is_tag_shortcut = false;
                if KNOWN_HTML_TAGS.contains(&name.as_str()) {
                    is_tag_shortcut = true;
                }

                let text_before = if !words.is_empty() {
                    let text = words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join(" ");
                    let text_span = Span::new(words[0].1.start, words.last().unwrap().1.end);
                    Some(TextNode::new(text, text_span))
                } else {
                    None
                };

                if is_tag_shortcut {
                    let element_span = Span::new(name_span.start, val_span.end);
                    let mut node = ElementNode::new(name, name_span, element_span);
                    node.children.push(Node::Text(TextNode::new(val, val_span)));
                    return Ok(MixedResult {
                        text_before,
                        node: Some(Node::Element(node)),
                        attr: None,
                    });
                } else {
                    let attr = Attribute::new(name, name_span, val, val_span);
                    return Ok(MixedResult {
                        text_before,
                        node: None,
                        attr: Some(attr),
                    });
                }
            }
        } else if self.check_token(|t| matches!(t, TokenType::LBrace)) {
            let (name, name_span) = words.pop().unwrap_or_else(|| (String::new(), Span::default()));

            // Nested pseudo-class block: hover { color: blue }
            if PSEUDO_CLASSES.contains(&name.as_str()) {
                if let Some(parent) = parent_tag {
                    let brace_pos = self.curr.as_ref().map(|t| t.span.start).unwrap_or(0);
                    let mut pseudo_node = ElementNode::new(name.clone(), name_span, Span::new(name_span.start, brace_pos));
                    self.match_token(|t| matches!(t, TokenType::LBrace));
                    self.parse_block(&mut pseudo_node)?;

                    let rules = parent.pseudo_rules.entry(name).or_default();
                    for attr in pseudo_node.attrs {
                        if let Some(existing) = rules.iter_mut().find(|(ek, _)| ek == &attr.key) {
                            existing.1 = attr.value;
                        } else {
                            rules.push((attr.key, attr.value));
                        }
                    }

                    let text_before = if !words.is_empty() {
                        let text = words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join(" ");
                        let text_span = Span::new(words[0].1.start, words.last().unwrap().1.end);
                        Some(TextNode::new(text, text_span))
                    } else {
                        None
                    };
                    return Ok(MixedResult {
                        text_before,
                        node: None,
                        attr: None,
                    });
                }
            }

            let brace_pos = self.curr.as_ref().map(|t| t.span.start).unwrap_or(0);
            let mut tag = ElementNode::new(name, name_span, Span::new(name_span.start, brace_pos));
            self.match_token(|t| matches!(t, TokenType::LBrace));

            if ["script", "style", "css", "javascript"].contains(&tag.name.as_str()) {
                let (val, end_brace_pos) = self.lexer.read_raw_until_matching_brace(brace_pos + 1);
                tag.span.end = end_brace_pos;
                self.curr = Some(self.lexer.next_token());
                self.peek = Some(self.lexer.next_token());
                self.peek2 = Some(self.lexer.next_token());
                tag.children.push(Node::Text(TextNode::new(val, Span::new(brace_pos + 1, end_brace_pos.saturating_sub(1)))));
            } else {
                self.parse_block(&mut tag)?;
            }

            let text_before = if !words.is_empty() {
                let text = words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join(" ");
                let text_span = Span::new(words[0].1.start, words.last().unwrap().1.end);
                Some(TextNode::new(text, text_span))
            } else {
                None
            };
            return Ok(MixedResult {
                text_before,
                node: Some(Node::Element(tag)),
                attr: None,
            });
        }

        if !words.is_empty() {
            let text = words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join(" ");
            let text_span = Span::new(words[0].1.start, words.last().unwrap().1.end);
            return Ok(MixedResult {
                text_before: None,
                node: Some(Node::Text(TextNode::new(text, text_span))),
                attr: None,
            });
        }

        Ok(MixedResult {
            text_before: None,
            node: None,
            attr: None,
        })
    }

    fn parse_block(&mut self, tag: &mut ElementNode) -> Result<(), String> {
        while !self.check_token(|t| matches!(t, TokenType::RBrace)) && !self.is_at_end() {
            if self.match_sep()
                || (self.match_token(|t| matches!(t, TokenType::Comma)).is_some() && !self.is_attr_comma())
            {
                continue;
            }

            let res = self.parse_mixed(Some(tag))?;
            let had_content = res.text_before.is_some() || res.node.is_some() || res.attr.is_some();

            if let Some(tb) = res.text_before {
                tag.children.push(Node::Text(tb));
            }

            if let Some(attr) = res.attr {
                tag.attrs.push(attr);
            } else if let Some(n) = res.node {
                tag.children.push(n);
            }

            if !had_content {
                self.advance();
            }
        }
        if let Some(rbrace) = self.match_token(|t| matches!(t, TokenType::RBrace)) {
            tag.span.end = rbrace.span.end;
        }
        Ok(())
    }

    fn parse_attribute_value(&mut self) -> (String, Span) {
        let is_paren = self.check_token(|t| matches!(t, TokenType::LParen));
        let is_brace = self.check_token(|t| matches!(t, TokenType::LBrace));

        if is_paren || is_brace {
            let is_paren_mode = is_paren;
            let start_tok = if is_paren {
                self.match_token(|t| matches!(t, TokenType::LParen)).unwrap()
            } else {
                self.match_token(|t| matches!(t, TokenType::LBrace)).unwrap()
            };
            let start_pos = start_tok.span.start;

            let mut res = String::new();
            let mut depth = 1;
            let mut end_pos = start_tok.span.end;

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
                    end_pos = tok.span.end;
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
                if let Some(rtok) = self.match_token(|t| matches!(t, TokenType::RParen)) {
                    end_pos = rtok.span.end;
                }
            } else {
                if let Some(rtok) = self.match_token(|t| matches!(t, TokenType::RBrace)) {
                    end_pos = rtok.span.end;
                }
            }

            while res.contains("  ") {
                res = res.replace("  ", " ");
            }
            return (res.trim().to_string(), Span::new(start_pos, end_pos));
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
                span,
            }) = self.advance()
            {
                words.push((w, span));
            }
        }

        if words.is_empty() {
            (String::new(), Span::default())
        } else {
            let start = words[0].1.start;
            let end = words.last().unwrap().1.end;
            let text = words.into_iter().map(|(w, _)| w).collect::<Vec<_>>().join(" ");
            (text, Span::new(start, end))
        }
    }
}

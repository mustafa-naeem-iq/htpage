use crate::token::{Token, TokenType};

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.pos >= self.chars.len() {
            return Token::new(TokenType::Eof, self.pos);
        }

        let ch = self.chars[self.pos];
        let start_pos = self.pos;

        match ch {
            '#' => {
                self.skip_comment();
                self.next_token()
            }
            '{' => {
                self.pos += 1;
                Token::new(TokenType::LBrace, start_pos)
            }
            '}' => {
                self.pos += 1;
                Token::new(TokenType::RBrace, start_pos)
            }
            '[' => {
                self.pos += 1;
                Token::new(TokenType::LBracket, start_pos)
            }
            ']' => {
                self.pos += 1;
                Token::new(TokenType::RBracket, start_pos)
            }
            '(' => {
                self.pos += 1;
                Token::new(TokenType::LParen, start_pos)
            }
            ')' => {
                self.pos += 1;
                Token::new(TokenType::RParen, start_pos)
            }
            '=' => {
                self.pos += 1;
                Token::new(TokenType::Assign, start_pos)
            }
            ':' => {
                if self.pos + 2 < self.chars.len()
                    && self.chars[self.pos + 1] == '/'
                    && self.chars[self.pos + 2] == '/'
                {
                    self.read_word()
                } else {
                    self.pos += 1;
                    Token::new(TokenType::Colon, start_pos)
                }
            }
            ',' => {
                self.pos += 1;
                Token::new(TokenType::Comma, start_pos)
            }
            ';' => {
                self.pos += 1;
                Token::new(TokenType::Sep(";".to_string()), start_pos)
            }
            '\n' => {
                self.pos += 1;
                Token::new(TokenType::Sep("\n".to_string()), start_pos)
            }
            _ => self.read_word(),
        }
    }

    pub fn read_raw_until_matching_brace(&mut self, start_pos: usize) -> String {
        self.pos = start_pos;
        let mut brace_count = 1;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    break;
                }
            }
            self.pos += 1;
        }

        let val: String = self.chars[start_pos..self.pos].iter().collect();
        self.pos += 1; // consume '}'
        val
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
            self.pos += 1;
        }
    }

    fn read_word(&mut self) -> Token {
        let start = self.pos;
        let mut word_str = String::new();

        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];

            // Support escape sequences (e.g. \{, \}, \#, \:, \,, \\)
            if ch == '\\' && self.pos + 1 < self.chars.len() {
                let next_ch = self.chars[self.pos + 1];
                if ['{', '}', '[', ']', '(', ')', '=', ':', ',', ';', '#', '\\'].contains(&next_ch) {
                    word_str.push(next_ch);
                    self.pos += 2;
                    continue;
                }
            }

            // Preserve URLs (e.g. https://)
            if ch == ':'
                && self.pos + 2 < self.chars.len()
                && self.chars[self.pos + 1] == '/'
                && self.chars[self.pos + 2] == '/'
            {
                word_str.push_str("://");
                self.pos += 3;
                continue;
            }

            if [' ', '\t', '\n', '\r', '{', '}', '[', ']', '(', ')', '=', ':', ',', ';', '#']
                .contains(&ch)
            {
                break;
            }

            word_str.push(ch);
            self.pos += 1;
        }

        Token::new(TokenType::Word(word_str), start)
    }
}

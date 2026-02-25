use crate::definitions::TokenType;
use crate::definitions::Token;
use crate::definitions::Pos;

pub struct Lexer {
    pub toks: Vec<Token>,
    pub pos: Pos,
    src: Vec<u8>,
    cur: usize,
    rune: u8,
}
impl Lexer {
    pub fn new(src: String) -> Self {
        let bytes = src.into_bytes();
        let first = *bytes.first().expect("Error: Provided source file is empty");
        Lexer { 
            toks: Vec::new(),
            pos: Pos { row: 0, col: 0 },
            src: bytes,
            cur: 0,
            rune: first,
        }
    }

    pub fn has_token(&self) -> bool {
        self.cur < self.toks.len() - 1
    }

    pub fn consume_token(&mut self) -> Token {
        let tok = self.toks.get(self.cur).expect("Error: Lexer failed to consume next token");
        self.cur += 1;
        tok.clone()
    }

    pub fn peek_token(&mut self) -> Token {
        let tok = self.toks.get(self.cur).expect("Error: Lexer failed to peek next token");
        tok.clone()
    }

    pub fn previous_token(&mut self) -> Token {
        let tok = self.toks.get(self.cur - 1).expect("Error: Lexer failed to peek previous token");
        tok.clone()
    }

    pub fn dump_remaining_tokens(&mut self) {
        eprintln!("Lexer Dump:");
        for i in self.cur..self.toks.len() {
            let tok = self.toks.get(i).expect("Error: Lexer failed to dump next token");
            eprintln!("{}", tok.val_str());
        }
    }

    pub fn tokenize(&mut self) {
        let mut lexeme: Vec<u8> = Vec::new();
        loop {
            match self.rune {
                b' ' | b'\n' => { // Delimiter and ignored
                    if !lexeme.is_empty() {
                        self.toks.push(Token { 
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                        });
                        lexeme.clear();
                    }
                },
                b';' | b'+' | b'-' | b'*' | b'/' | b'=' | b'(' | b')' => { // Delimiter
                    if !lexeme.is_empty() {
                        self.toks.push(Token { 
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                        });
                        lexeme.clear();
                    }

                    // Also have to push on the delimiter token
                    lexeme.push(self.rune);
                    self.toks.push(Token { 
                        kind: TokenType::None,
                        val: lexeme.clone(),
                        pos: Pos { row: self.pos.row, col: self.pos.col },
                    });
                    lexeme.clear();
                },
                _ => { // Other
                    lexeme.push(self.rune);
                }
            }

            // Advance cursor, position, and rune until end of file
            if !self.advance() { break; }
        }

        // Push the last lexeme if it contains characters
        if !lexeme.is_empty() {
            self.toks.push(Token { 
                kind: TokenType::None,
                val: lexeme.clone(),
                pos: Pos { row: self.pos.row, col: self.pos.col - 1 },
            });
        }

        self.cur = 0;
    }

    fn advance(&mut self) -> bool {
        self.cur += 1;
        self.pos.col += 1;
        if self.cur >= self.src.len() {
            self.rune = 0;
            false
        } else {
            if self.rune == b'\n' {
                self.pos.row += 1;
                self.pos.col = 0;
            }
            self.rune = *self.src.get(self.cur).unwrap_or_else(|| panic!("{} Error: Failed to advance to next rune", self.pos));
            true
        }
    }

    pub fn lex(&mut self) {
        for tok in &mut self.toks {
            if tok.val.is_empty() {
                panic!("{} Error: Cannot have an empty token", tok.pos);
            }

            let first: &u8 = tok.val.first().unwrap_or_else(|| panic!("{} Error: Failed to get first char in token", tok.pos));
            match first { // First try to match using first byte
                b'+' => tok.kind = TokenType::OpPlus,
                b'-' => tok.kind = TokenType::OpMinus,
                b'*' => tok.kind = TokenType::OpMul,
                b'/' => tok.kind = TokenType::OpDiv,
                b':' => tok.kind = TokenType::OpAssign,
                b'(' => tok.kind = TokenType::OpenParen,
                b')' => tok.kind = TokenType::CloseParen,
                b'=' => tok.kind = TokenType::Equal,
                b'~' => tok.kind = TokenType::NotEqual,
                b'>' => tok.kind = TokenType::GreaterThan,
                b'<' => tok.kind = TokenType::LessThan,
                b'{' => tok.kind = TokenType::OpenScope,
                b'}' => tok.kind = TokenType::CloseScope,
                b';' => tok.kind = TokenType::End,
                _    => { // Then try to match using the whole word
                    match tok.val_str().as_str() {
                        ">="     => tok.kind = TokenType::GreaterEqual,
                        "<="     => tok.kind = TokenType::LessEqual,
                        "and"    => tok.kind = TokenType::LogicalAnd,
                        "or"     => tok.kind = TokenType::LogicalOr,
                        "exit"   => tok.kind = TokenType::KeywordExit,
                        "func"   => tok.kind = TokenType::KeywordFunctionDecl,
                        "dump"   => tok.kind = TokenType::KeywordDebugDump,
                        _ => { // Then match variable contents of words
                            if tok.val.iter().all(|c| c.is_ascii_digit()) {
                                tok.kind = TokenType::LiteralInt;
                            } else if first.is_ascii_alphabetic() {
                                tok.kind = TokenType::Identifier;
                            } else {
                                panic!("{} Error: Invalid token `{}`", tok.pos, tok.val_str());
                            }
                        }
                    }
                }
            }
        }
    }
}

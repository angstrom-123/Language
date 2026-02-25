use crate::definitions::TokenType;
use crate::definitions::Token;
use crate::definitions::Pos;

#[derive(PartialEq)]
enum TokenParseType {
    Letter,
    Number,
    Operator,
    End,
    Scope
}

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
        let mut parse_type: Vec<TokenParseType> = Vec::new();
        for tok in &mut self.toks {
            let s: String = tok.val_str();

            // Find the types of each character making up the token's value
            for byte in &tok.val {
                match byte {
                    b'+' | b'-' | b'*' | b'/' | b'=' | b'(' | b')' => parse_type.push(TokenParseType::Operator),
                    b'{' | b'}' => parse_type.push(TokenParseType::Scope),
                    b';' => parse_type.push(TokenParseType::End),
                    b'0'..=b'9' => parse_type.push(TokenParseType::Number),
                    b'A'..=b'z' => parse_type.push(TokenParseType::Letter),
                    _ => panic!("{} Error: Invalid first character in token `{}`", tok.pos, s),
                }
            }

            // Get the category of the token based on its character types.
            let mut it = parse_type.iter();
            let n: usize = it.len();
            let first: &TokenParseType = it.next().unwrap_or_else(|| panic!("{} Error: Token must have at least 1 parse type", tok.pos));
            match first {
                TokenParseType::Operator => {
                    assert!(n == 1, "{} Error: Operator type tokens must have a length of 1", tok.pos);
                    match s.as_str() {
                        "+" => tok.kind = TokenType::OpPlus,
                        "-" => tok.kind = TokenType::OpMinus,
                        "*" => tok.kind = TokenType::OpMul,
                        "/" => tok.kind = TokenType::OpDiv,
                        "=" => tok.kind = TokenType::OpAssign,
                        "(" => tok.kind = TokenType::OpOpenParen,
                        ")" => tok.kind = TokenType::OpCloseParen,
                        _ => panic!("{} Error: Invalid value for operator token `{}`", tok.pos, s),
                    }
                },
                TokenParseType::Scope => {
                    match s.as_str() {
                        "{" => tok.kind = TokenType::OpenScope,
                        "}" => tok.kind = TokenType::CloseScope,
                        _ => panic!("{} Error: Invalid value for scope token `{}`", tok.pos, s),
                    }
                },
                TokenParseType::End => {
                    assert!(n == 1, "{} Error: Invalid value for end token `{}`", tok.pos, s);
                    tok.kind = TokenType::End;
                },
                TokenParseType::Number => {
                    for next in it { 
                        if *next != TokenParseType::Number {
                            panic!("{} Error: Unexpected letter in number `{}`", tok.pos, s);
                        }
                    }
                    tok.kind = TokenType::LiteralInt;
                },
                TokenParseType::Letter => {
                    for next in it { 
                        if matches!(next, TokenParseType::End | TokenParseType::Operator) {
                            panic!("{} Error: Unexpected special character in word `{}`", tok.pos, s);
                        }
                    }

                    // Match against keywords
                    match s.as_str() {
                        "return" => tok.kind = TokenType::KeywordReturn,
                        "dump"   => tok.kind = TokenType::KeywordDebugDump,
                        _        => tok.kind = TokenType::Identifier,
                    }
                }
            }

            parse_type.clear();
        }
    }
}

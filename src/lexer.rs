use crate::definitions::TokenType;
use crate::definitions::Token;
use crate::definitions::Pos;

#[derive(PartialEq)]
enum TokenParseType {
    Letter,
    Number,
    Operator,
    End
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
                b';' | b'+' | b'-' | b'*' | b'/' | b'=' => { // Delimiter
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
            let s: String = String::from_utf8(tok.val.clone()).unwrap_or_else(|_| panic!("{} Error: Failed to convert token value to string", tok.pos));

            // Find the types of each character making up the token's value
            for byte in &tok.val {
                match byte {
                    b'+' | b'-' | b'*' | b'/' | b'=' => parse_type.push(TokenParseType::Operator),
                    b';'=> parse_type.push(TokenParseType::End),
                    b'0'..=b'9' => parse_type.push(TokenParseType::Number),
                    b'A'..=b'z' => parse_type.push(TokenParseType::Letter),
                    _ => unreachable!("{} Error: Invalid first character in token", tok.pos),
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
                        _ => unreachable!("Error: Invalid value for operator token"),
                    }
                },
                TokenParseType::End => {
                    assert!(n == 1, "{} Error: End type tokens must have a length of 1", tok.pos);
                    tok.kind = TokenType::End;
                },
                TokenParseType::Number => {
                    for next in it { 
                        if *next != TokenParseType::Number {
                            panic!("{} Error: Expression starting with number must only contain numbers", tok.pos);
                        }
                    }
                    tok.kind = TokenType::LiteralInt;
                },
                TokenParseType::Letter => {
                    for next in it { 
                        if matches!(next, TokenParseType::End | TokenParseType::Operator) {
                            panic!("{} Error: Expression starting with letter must not contain special characters", tok.pos);
                        }
                    }

                    // Match against keywords
                    match s.as_str() {
                        "return" => tok.kind = TokenType::KeywordReturn,
                        "dump"   => tok.kind = TokenType::KeywordDebugDump,
                        "let"    => tok.kind = TokenType::KeywordDeclare,
                        _        => tok.kind = TokenType::Identifier,
                    }
                }
            }

            parse_type.clear();
        }
    }
}

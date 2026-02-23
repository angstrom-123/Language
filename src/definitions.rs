use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenType {
    None,
    End,
    OpPlus,
    OpMinus,
    OpMul,
    OpDiv,
    OpAssign,
    KeywordReturn,
    KeywordDebugDump,
    KeywordDeclare,
    Identifier,
    LiteralInt,
}
impl TokenType {
    pub fn precedence(&self) -> i32 {
        match self {
            TokenType::OpPlus     => 5,
            TokenType::OpMinus    => 5,
            TokenType::LiteralInt => 0,
            TokenType::Identifier => 0,
            TokenType::OpMul      => 10,
            TokenType::OpDiv      => 10,
            _                     => -1,
        }
    }
}

#[derive(Clone)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}
impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", self.row, self.col)
    }
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenType,
    pub val: Vec<u8>,
    pub pos: Pos,
}
impl Token {
    pub fn val_str(&self) -> String {
        String::from_utf8(self.val.clone()).expect("Error: Failed to convert token value to string")
    }
}

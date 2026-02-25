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
    OpenParen,
    CloseParen,
    OpenScope,
    CloseScope,
    KeywordFunctionDecl,
    KeywordReturn,
    KeywordDebugDump,
    Identifier,
    LiteralInt,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    LogicalOr,
    LogicalAnd,
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

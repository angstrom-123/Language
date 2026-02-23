use crate::definitions::Pos;
use crate::definitions::Token;
use crate::definitions::TokenType;
use crate::lexer::Lexer;

#[derive(Debug)]
#[derive(Clone)]
pub enum ExprType {
    Program,
    Return,
    DebugDump,
    Declare,
    Assign,
    BinOp,
    Variable,
    Literal,
}

#[derive(Clone)]
pub struct ParseNode {
    pub kind: ExprType,
    pub tok: Token,
    pub children: Vec<ParseNode>,
}
impl ParseNode {
    pub fn dump(&self, _depth: usize) {
        let tok_str: String = String::from_utf8(self.tok.val.clone()).expect("Error: Failed to convert token value to string");
        eprintln!("{:padding$}{:?} `{}`", "", self.kind, tok_str, padding = _depth);
        for child in &self.children {
            child.dump(_depth + 4);
        }
    }

    fn new_program(prog_name: String, prog: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: ExprType::Program,
            tok: Token {
                kind: TokenType::None,
                val: prog_name.into_bytes(),
                pos: Pos { row: usize::MAX, col: usize::MAX },
            },
            children: prog
        }
    }

    fn new_return(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: ExprType::Return,
            tok,
            children: vec![rhs],
        }
    }

    fn new_debug_dump(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: ExprType::DebugDump,
            tok,
            children: vec![rhs],
        }
    }

    fn new_declare(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: ExprType::Declare,
            tok,
            children: vec![rhs],
        }
    }

    fn new_assign(tok: Token, lhs: ParseNode, rhs: ParseNode) -> Self {
        ParseNode {
            kind: ExprType::Assign,
            tok,
            children: vec![lhs, rhs],
        }
    }

    fn new_bin_op(tok: Token, lhs: ParseNode, rhs: ParseNode) -> Self {
        ParseNode {
            kind: ExprType::BinOp,
            tok,
            children: vec![lhs, rhs],
        }
    }

    fn new_variable(tok: Token) -> Self {
        ParseNode {
            kind: ExprType::Variable,
            tok,
            children: Vec::new(),
        }
    }

    fn new_literal(tok: Token) -> Self {
        ParseNode {
            kind: ExprType::Literal,
            tok,
            children: Vec::new(),
        }
    }
}

pub struct ParseTree {
    pub root: ParseNode
}
impl ParseTree {
    pub fn new(prog_name: String) -> Self {
        ParseTree { root: ParseNode::new_program(prog_name, Vec::new()) }
    }

    pub fn construct(&mut self, lexer: &mut Lexer) {
        let mut children: Vec<ParseNode> = Vec::new();
        while lexer.has_token() {
            children.push(self.parse_expr(lexer, -1));
        }
        self.root.children = children;
    }

    pub fn dump(&self) {
        self.root.dump(0);
    }

    pub fn traverse(&mut self, res: &mut Vec<ParseNode>) {
        self.post_order(self.root.clone(), res);
    }

    fn post_order(&mut self, curr: ParseNode, res: &mut Vec<ParseNode>) {
        for node in &curr.children {
            self.post_order(node.clone(), res);
        }

        res.push(curr);
    }

    fn parse_sub_expr(&mut self, lexer: &mut Lexer, precedence: i32) -> ParseNode {
        let mut tok: Token = lexer.consume_token();
        if tok.kind != TokenType::LiteralInt {
            panic!("{} Error: Invalid start to subexpression `{}`", tok.pos, tok.val_str());
        }

        let sub_expr: ParseNode = ParseNode::new_literal(tok);
        tok = lexer.peek_token();
        match tok.kind {
            TokenType::OpMul | TokenType::OpDiv | TokenType::OpPlus => {
                let p: i32 = TokenType::precedence(&tok.kind);
                if p >= precedence {
                    lexer.consume_token();
                    ParseNode::new_bin_op(tok, sub_expr, self.parse_sub_expr(lexer, p))
                } else {
                    sub_expr
                }
            },
            TokenType::OpMinus => {
                let p: i32 = TokenType::precedence(&tok.kind);
                if p > precedence {
                    lexer.consume_token();
                    ParseNode::new_bin_op(tok, sub_expr, self.parse_sub_expr(lexer, p))
                } else {
                    sub_expr
                }
            },
            TokenType::End => {
                sub_expr
            },
            _ => panic!("{} Error: Expected operator or end but got `{}`", tok.pos, tok.val_str()),
        }
    }

    fn parse_expr(&mut self, lexer: &mut Lexer, precedence: i32) -> ParseNode {
        let mut tok: Token = lexer.consume_token();
        let orig_tok = &tok.clone();
        let mut sub_expr = self.parse_sub_expr(lexer, precedence);
        loop {
            tok = lexer.peek_token();
            match tok.kind {
                TokenType::OpMul | TokenType::OpDiv | TokenType::OpPlus | TokenType::OpMinus => {
                    tok = lexer.consume_token();
                    let p: i32 = TokenType::precedence(&tok.kind);
                    sub_expr = ParseNode::new_bin_op(tok, sub_expr.clone(), self.parse_sub_expr(lexer, p));
                },
                TokenType::End => {
                    lexer.consume_token();
                    match orig_tok.kind {
                        TokenType::KeywordReturn => {
                            return ParseNode::new_return(orig_tok.clone(), sub_expr.clone());
                        },
                        TokenType::KeywordDebugDump => {
                            return ParseNode::new_debug_dump(orig_tok.clone(), sub_expr.clone());
                        },
                        _ => panic!("Currently only valid start to expr is `return`"),
                    }
                },
                _ => panic!("{} Error: Expected operator or end but got `{}`", tok.pos, tok.val_str()),
            };
        }
    }
}

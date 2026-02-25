use crate::definitions::Pos;
use crate::definitions::Token;
use crate::definitions::TokenType;
use crate::lexer::Lexer;

#[derive(Debug)]
#[derive(Clone)]
pub enum NodeType {
    Program,
    Return,
    FuncDecl,
    DebugDump,
    BinOp,
    UnOp,
    Literal,
}

#[derive(Clone)]
pub struct ParseNode {
    pub kind: NodeType,
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
            kind: NodeType::Program,
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
            kind: NodeType::Return,
            tok,
            children: vec![rhs],
        }
    }

    fn new_debug_dump(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::DebugDump,
            tok,
            children: vec![rhs],
        }
    }

    fn new_bin_op(tok: Token, lhs: ParseNode, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::BinOp,
            tok,
            children: vec![lhs, rhs],
        }
    }

    fn new_un_op(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::UnOp,
            tok,
            children: vec![rhs],
        }
    }

    fn new_literal(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::Literal,
            tok,
            children: Vec::new(),
        }
    }

    fn new_func_decl(ident_tok: Token, body: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::FuncDecl,
            tok: ident_tok,
            children: body,
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
            children.push(self.parse_statement(lexer));
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

    fn parse_factor(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut tok: Token = lexer.consume_token();
        match tok.kind {
            TokenType::LiteralInt => ParseNode::new_literal(tok),
            TokenType::OpMinus => { // Unary minus
                let factor: ParseNode = self.parse_factor(lexer);
                ParseNode::new_un_op(tok, factor)
            },
            TokenType::OpenParen => {
                let expression: ParseNode = self.parse_expression(lexer);
                tok = lexer.consume_token();
                if tok.kind != TokenType::CloseParen {
                    panic!("{} Error: Expected `)` but got `{}`", tok.pos, tok.val_str());
                }
                expression
            },
            _ => panic!("{} Error: Invalid factor `{}`", tok.pos, tok.val_str())
        }
    }

    fn parse_term(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut factor: ParseNode = self.parse_factor(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpMul | TokenType::OpDiv) {
            lexer.consume_token();
            let next_factor: ParseNode = self.parse_factor(lexer);
            factor = ParseNode::new_bin_op(tok, factor, next_factor);
            tok = lexer.peek_token();
        }

        factor
    }

    fn parse_expression(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut term: ParseNode = self.parse_term(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpPlus | TokenType::OpMinus) {
            lexer.consume_token();
            let next_term: ParseNode = self.parse_term(lexer);
            term = ParseNode::new_bin_op(tok, term, next_term);
            tok = lexer.peek_token();
        }

        term
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> ParseNode {
        let tok: Token = lexer.consume_token();
        match tok.kind {
            TokenType::KeywordReturn => {
                let expression: ParseNode = self.parse_expression(lexer);
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_return(tok, expression)
            },
            TokenType::KeywordDebugDump => {
                let expression: ParseNode = self.parse_expression(lexer);
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_debug_dump(tok, expression)
            },
            TokenType::KeywordFunctionDecl => {
                let name_tok: Token = lexer.consume_token();
                if name_tok.kind != TokenType::Identifier {
                    panic!("{} Error: Expected identifier but got `{}`", name_tok.pos, name_tok.val_str());
                }

                let mut next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::OpenScope {
                    panic!("{} Error: Expected `{{` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                let mut body: Vec<ParseNode> = Vec::new();
                while next_tok.kind != TokenType::CloseScope {
                    body.push(self.parse_statement(lexer));
                    next_tok = lexer.peek_token();
                }
                lexer.consume_token();
                ParseNode::new_func_decl(name_tok, body)
            },
            _ => panic!("{} Error: Unknown statement `{}`", tok.pos, tok.val_str()),
        }
    }
}

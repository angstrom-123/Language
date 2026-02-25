use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenType;
use crate::lexer::Pos;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum NodeType {
    Program,
    Exit,
    FuncDecl,
    FuncCall,
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
        eprintln!("{:padding$}\x1b[94m* {:?}\x1b[0m (\x1b[92m{:?}\x1b[0m: `{}`)", "", self.kind, self.tok.kind, tok_str, padding = _depth);
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

    fn new_exit(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::Exit,
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

    fn new_func_call(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::FuncCall,
            tok,
            children: vec![],
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
            children.push(self.parse_function(lexer));
        }
        self.root.children = children;
    }

    pub fn dump(&self) {
        self.root.dump(0);
    }

    pub fn post_order(&self, curr: ParseNode, res: &mut Vec<ParseNode>) {
        for node in &curr.children {
            self.post_order(node.clone(), res);
        }

        res.push(curr);
    }

    /*
     * Production Rules:
     *
     * <program>   ::= { <function> }
     * <function>  ::= "func" <id> "{" { <statement> } "}"
     * <statement> ::= "dump" <expression> ";" 
     *               | "exit" <expression> ";" 
     *               | <id> "(" ")" ";"
     *               | "if" <log_expr> "{" { <statement> } "}"
     * <log_expr>  ::= <and_expr> { "||" <and_expr> }
     * <and_expr>  ::= <equ_expr> { "&&" <equ_expr> }
     * <equ_expr>  ::= <rel_expr> { ("==" | "~=") <rel_expr> }
     * <rel_expr>  ::= <add_expr> { ("<" | ">" | "<=" | ">=") <add_expr> }
     * <add_expr>  ::= <term> { ("+" | "-") <term> }
     * <term>      ::= <factor> { ("*" | "/") <factor> }
     * <factor>    ::= "(" <log_expr> ")" | <unary_op> <factor> | <int>
     * <unary_op>  ::= "-"
     */

    fn parse_factor(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut tok: Token = lexer.consume_token();
        match tok.kind {
            TokenType::LiteralInt => ParseNode::new_literal(tok),
            TokenType::OpMinus => { // Unary minus
                let factor: ParseNode = self.parse_factor(lexer);
                ParseNode::new_un_op(tok, factor)
            },
            TokenType::OpenParen => {
                let expression: ParseNode = self.parse_add_expr(lexer);
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

    fn parse_add_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
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

    fn parse_log_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut and: ParseNode = self.parse_and_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpLogicalOr) {
            lexer.consume_token();
            let next_and: ParseNode = self.parse_and_expr(lexer);
            and = ParseNode::new_bin_op(tok, and, next_and);
            tok = lexer.peek_token();
        }

        and
    }

    fn parse_and_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut equ: ParseNode = self.parse_equ_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpLogicalAnd) {
            lexer.consume_token();
            let next_equ: ParseNode = self.parse_equ_expr(lexer);
            equ = ParseNode::new_bin_op(tok, equ, next_equ);
            tok = lexer.peek_token();
        }

        equ
    }

    fn parse_equ_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut rel: ParseNode = self.parse_rel_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpEqual | TokenType::OpNotEqual) {
            lexer.consume_token();
            let next_rel: ParseNode = self.parse_rel_expr(lexer);
            rel = ParseNode::new_bin_op(tok, rel, next_rel);
            tok = lexer.peek_token();
        }

        rel
    }

    fn parse_rel_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut add: ParseNode = self.parse_add_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpGreaterThan | TokenType::OpGreaterEqual | TokenType::OpLessThan | TokenType::OpLessEqual) {
            lexer.consume_token();
            let next_add: ParseNode = self.parse_add_expr(lexer);
            add = ParseNode::new_bin_op(tok, add, next_add);
            tok = lexer.peek_token();
        }

        add
    }

    fn parse_function(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut tok: Token = lexer.consume_token();
        if tok.kind != TokenType::KeywordFunctionDecl {
            panic!("{} Error: Expected function declaration but got `{}`", tok.pos, tok.val_str());
        }
        tok = lexer.consume_token();
        if tok.kind != TokenType::Identifier {
            panic!("{} Error: Expected identifier but got `{}`", tok.pos, tok.val_str());
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
        ParseNode::new_func_decl(tok, body)
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> ParseNode {
        let tok: Token = lexer.consume_token();
        match tok.kind {
            TokenType::KeywordExit => {
                let expression: ParseNode = self.parse_add_expr(lexer);
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_exit(tok, expression)
            },
            TokenType::KeywordDebugDump => {
                let expression: ParseNode = self.parse_add_expr(lexer);
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_debug_dump(tok, expression)
            },
            TokenType::Identifier => {
                let mut next_tok: Token = lexer.consume_token();
                match next_tok.kind {
                    TokenType::OpenParen => {
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::CloseParen {
                            panic!("{} Error: Expected `)` but got `{}`", next_tok.pos, next_tok.val_str());
                        }
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::End {
                            panic!("{} Error: Expected `; but got `{}`", next_tok.pos, next_tok.val_str());
                        }
                        ParseNode::new_func_call(tok)
                    },
                    _ => {
                        panic!("{} Error: Unexpected identifier `{}`", tok.pos, tok.val_str());
                    }
                }
            },
            _ => panic!("{} Error: Unknown statement `{}`", tok.pos, tok.val_str()),
        }
    }
}

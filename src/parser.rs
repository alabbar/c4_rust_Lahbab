//! A recursive-descent parser producing an AST for C4’s subset.

use crate::lexer::Token;
use crate::error::{Error, Result};

/// An AST supporting `int f() { … }`, `return`, and binary expressions.
#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    },
    Return(Box<ASTNode>),
    Number(i64),
    BinaryOp {
        op: String,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Build a parser over a token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse `Program = Func* EOF`.
    pub fn parse_program(&mut self) -> Result<ASTNode> {
        let mut funcs = Vec::new();
        while self.peek() != Token::EOF {
            funcs.push(self.parse_function()?);
        }
        Ok(ASTNode::Program(funcs))
    }

    fn parse_function(&mut self) -> Result<ASTNode> {
        // expect “int”
        match self.next_token() {
            Token::Keyword(ref k) if k == "int" => {}
            other => return Err(Error::UnexpectedToken(format!("{:?}", other))),
        }
        // expect name
        let name = match self.next_token() {
            Token::Id(s) => s,
            other        => return Err(Error::UnexpectedToken(format!("{:?}", other))),
        };
        // expect "()"
        self.expect_op("(")?;
        self.expect_op(")")?;
        // parse block
        let body = self.parse_block()?;
        Ok(ASTNode::Function { name, params: Vec::new(), body })
    }

    fn parse_block(&mut self) -> Result<Vec<ASTNode>> {
        self.expect_op("{")?;
        let mut stmts = Vec::new();
        while self.peek() != Token::Operator("}".into()) && self.peek() != Token::EOF {
            stmts.push(self.parse_statement()?);
        }
        self.expect_op("}")?;
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<ASTNode> {
        if let Token::Keyword(ref kw) = self.peek() {
            if kw == "return" {
                return self.parse_return();
            }
        }
        Err(Error::InvalidSyntax(format!("Unexpected statement start: {:?}", self.peek())))
    }

    fn parse_return(&mut self) -> Result<ASTNode> {
        self.next_token(); // consume 'return'
        let expr = self.parse_expr()?;
        self.expect_op(";")?;
        Ok(ASTNode::Return(Box::new(expr)))
    }

    fn parse_expr(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_primary()?;
        while let Token::Operator(ref op) = self.peek() {
            if ["+", "-", "*", "/", "%"].contains(&op.as_str()) {
                let op = op.clone();
                self.next_token();
                let rhs = self.parse_primary()?;
                node = ASTNode::BinaryOp { op, left: Box::new(node), right: Box::new(rhs) };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<ASTNode> {
        match self.next_token() {
            Token::Num(n) => Ok(ASTNode::Number(n)),
            Token::Operator(ref s) if s == "(" => {
                let inner = self.parse_expr()?;
                self.expect_op(")")?;
                Ok(inner)
            }
            other => Err(Error::UnexpectedToken(format!("{:?}", other))),
        }
    }

    fn expect_op(&mut self, op: &str) -> Result<()> {
        match self.next_token() {
            Token::Operator(s) if s == op => Ok(()),
            other                         => Err(Error::UnexpectedToken(format!("{:?}", other))),
        }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF)
    }

    fn next_token(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF);
        self.pos += 1;
        t
    }
}

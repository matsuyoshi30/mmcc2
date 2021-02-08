use std::process;

use crate::tokenize::{Token, TokenKind};
use crate::types::{Type, TypeKind};

#[derive(PartialEq)]
pub enum NodeKind {
    NdAdd,
    NdSub,
    NdMul,
    NdDiv,
    NdNum,
    NdMt, // more than >
    NdLt, // less than <
    NdOm, // or more >=
    NdOl, // or less <=
    NdEq, // equal
    NdNe, // not equal
    NdAs,
    NdLv,
    NdIf,
    NdWhile,
    NdFor,
    NdBlock,
    NdFunc,
    NdRt,
}

impl Default for NodeKind {
    fn default() -> Self {
        NodeKind::NdNum
    }
}

#[derive(Default)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: u32,
    pub offset: usize,
    pub cond: Option<Box<Node>>,
    pub then: Option<Box<Node>>,
    pub els: Option<Box<Node>>,
    pub preop: Option<Box<Node>>,
    pub postop: Option<Box<Node>>,
    pub blocks: Vec<Node>,
    pub funcname: String,
    pub args: Vec<Node>,
}

impl Node {
    fn new_node(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            kind: kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            ..Default::default()
        }
    }

    fn new_node_lv(offset: usize) -> Self {
        Self {
            kind: NodeKind::NdLv,
            offset: offset,
            ..Default::default()
        }
    }

    fn new_node_return(lhs: Box<Node>) -> Self {
        Self {
            kind: NodeKind::NdRt,
            lhs: Some(lhs),
            ..Default::default()
        }
    }

    fn new_node_num(val: u32) -> Self {
        Self {
            val: val,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct LVar {
    pub ty: TypeKind,
    pub name: String,
    pub offset: usize,
}

pub struct Function {
    pub ty: TypeKind,
    pub name: String,
    pub paramnum: usize,
    pub locals: Vec<LVar>,
    pub body: Vec<Node>,
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
    temp_locals: Vec<LVar>,
    pub functions: Vec<Function>,
}

impl<'a> Parser<'a> {
    fn find_lvar(&mut self, name: String) -> usize {
        for local in &self.temp_locals {
            if local.name == name {
                return local.offset;
            }
        }

        0
    }

    fn funcargs(&mut self) -> Vec<Node> {
        let mut args = vec![];
        if self.tokens[self.pos].op == ")" {
            self.pos += 1;
            return args;
        }

        args.push(self.add());
        while self.tokens[self.pos].op == "," {
            self.pos += 1;
            args.push(self.add());
        }
        self.expect(")");

        args
    }

    fn primary(&mut self) -> Node {
        if self.tokens[self.pos].op == "(" {
            self.pos += 1;
            let node = self.expr();
            if self.tokens[self.pos].op != ")" {
                eprintln!("unexpected token: {}", self.tokens[self.pos].op);
                process::exit(1);
            }
            self.pos += 1;
            return node;
        }

        if self.tokens[self.pos].kind == TokenKind::TkIdent {
            let name = &self.tokens[self.pos].op;

            self.pos += 1;
            if self.tokens[self.pos].op == "(" {
                self.pos += 1;
                let node = Node {
                    kind: NodeKind::NdFunc,
                    funcname: name.to_string(),
                    args: self.funcargs(),
                    ..Default::default()
                };

                return node;
            } else {
                let offset = self.find_lvar(name.to_string());
                return Node::new_node_lv(offset);
            }
        }

        self.pos += 1;
        Node::new_node_num(self.tokens[self.pos - 1].val)
    }

    fn unary(&mut self) -> Node {
        if self.tokens[self.pos].op == "+" {
            self.pos += 1;
            return self.unary();
        }
        if self.tokens[self.pos].op == "-" {
            self.pos += 1;
            return Node::new_node(
                NodeKind::NdSub,
                Box::new(Node::new_node_num(0)),
                Box::new(self.unary()),
            );
        }

        self.primary()
    }

    fn mul(&mut self) -> Node {
        let mut lhs = self.unary();

        loop {
            match self.tokens[self.pos].op.as_str() {
                "*" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdMul, Box::new(lhs), Box::new(self.unary()));
                }
                "/" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdDiv, Box::new(lhs), Box::new(self.unary()));
                }
                _ => {
                    break;
                }
            }
        }

        lhs
    }

    fn add(&mut self) -> Node {
        let mut lhs = self.mul();

        loop {
            match self.tokens[self.pos].op.as_str() {
                "+" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdAdd, Box::new(lhs), Box::new(self.mul()));
                }
                "-" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdSub, Box::new(lhs), Box::new(self.mul()));
                }
                _ => {
                    break;
                }
            }
        }

        lhs
    }

    fn relational(&mut self) -> Node {
        let mut lhs = self.add();

        loop {
            match self.tokens[self.pos].op.as_str() {
                ">" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdMt, Box::new(lhs), Box::new(self.add()));
                }
                "<" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdLt, Box::new(lhs), Box::new(self.add()));
                }
                ">=" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdOm, Box::new(lhs), Box::new(self.add()));
                }
                "<=" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdOl, Box::new(lhs), Box::new(self.add()));
                }
                _ => {
                    break;
                }
            }
        }

        lhs
    }

    fn equality(&mut self) -> Node {
        let mut lhs = self.relational();

        loop {
            match self.tokens[self.pos].op.as_str() {
                "==" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdEq, Box::new(lhs), Box::new(self.mul()));
                }
                "!=" => {
                    self.pos += 1;
                    lhs = Node::new_node(NodeKind::NdNe, Box::new(lhs), Box::new(self.mul()));
                }
                _ => {
                    break;
                }
            }
        }

        lhs
    }

    fn assign(&mut self) -> Node {
        let mut lhs = self.equality();

        if self.tokens[self.pos].op.as_str() == "=" {
            self.pos += 1;
            lhs = Node::new_node(NodeKind::NdAs, Box::new(lhs), Box::new(self.assign()));
        }

        lhs
    }

    fn expr(&mut self) -> Node {
        return self.assign();
    }

    fn stmt(&mut self) -> Node {
        let mut node;

        if self.tokens[self.pos].op == "{" {
            node = Node {
                kind: NodeKind::NdBlock,
                blocks: vec![],
                ..Default::default()
            };
            self.pos += 1;
            while self.tokens[self.pos].op != "}" {
                node.blocks.push(self.stmt());
            }
            self.pos += 1;
            return node;
        }

        let ty = Type::consume_type(&self.tokens[self.pos].op);
        if ty.kind != TypeKind::TyNone {
            self.pos += 1;
            let name = &self.tokens[self.pos].op;
            let offset = (self.temp_locals.len() + 1) * 8;
            let lvar = LVar {
                ty: ty.kind,
                name: name.to_string(),
                offset: offset,
            };

            self.pos += 1;
            self.expect(";");

            self.temp_locals.push(lvar);
            return Node::new_node_lv(offset);
        }

        if self.tokens[self.pos].op == "return" {
            self.pos += 1;
            node = Node::new_node_return(Box::new(self.expr()));
        } else if self.tokens[self.pos].op == "if" {
            node = Node {
                kind: NodeKind::NdIf,
                ..Default::default()
            };
            self.pos += 1;
            self.expect("(");
            node.cond = Some(Box::new(self.expr()));
            self.expect(")");
            node.then = Some(Box::new(self.stmt()));
            if self.tokens[self.pos].op == "else" {
                self.pos += 1;
                node.els = Some(Box::new(self.stmt()));
            }

            return node;
        } else if self.tokens[self.pos].op == "while" {
            node = Node {
                kind: NodeKind::NdWhile,
                ..Default::default()
            };
            self.pos += 1;
            self.expect("(");
            node.cond = Some(Box::new(self.expr()));
            self.expect(")");
            node.then = Some(Box::new(self.stmt()));

            return node;
        } else if self.tokens[self.pos].op == "for" {
            node = Node {
                kind: NodeKind::NdFor,
                ..Default::default()
            };
            self.pos += 1;
            self.expect("(");
            if self.tokens[self.pos].op != ";" {
                node.preop = Some(Box::new(self.expr()));
            }
            self.expect(";");
            if self.tokens[self.pos].op != ";" {
                node.cond = Some(Box::new(self.expr()));
            }
            self.expect(";");
            if self.tokens[self.pos].op != ")" {
                node.postop = Some(Box::new(self.expr()));
            }
            self.expect(")");
            node.then = Some(Box::new(self.stmt()));

            return node;
        } else {
            node = self.expr();
        }

        if self.tokens[self.pos].op.as_str() != ";" {
            eprintln!("expected ';'");
            process::exit(1)
        }
        self.pos += 1;

        node
    }

    fn function(&mut self) -> Function {
        let ty = Type::consume_type(&self.tokens[self.pos].op);
        self.pos += 1;
        let name = self.expect_ident();
        let mut func = Function {
            ty: ty.kind,
            name: name,
            paramnum: 0,
            locals: vec![],
            body: vec![],
        };

        self.expect("(");
        if self.tokens[self.pos].op == ")" {
            self.pos += 1;
        } else {
            let mut ty = Type::consume_type(&self.tokens[self.pos].op);
            self.pos += 1;
            let mut name = self.tokens[self.pos].op.to_string();
            let lvar = LVar {
                ty: ty.kind,
                name: name,
                offset: (self.temp_locals.len() + 1) * 8,
            };
            self.temp_locals.push(lvar);
            self.pos += 1;

            while self.tokens[self.pos].op == "," {
                self.pos += 1;
                ty = Type::consume_type(&self.tokens[self.pos].op);
                self.pos += 1;
                name = self.tokens[self.pos].op.to_string();
                let lvar = LVar {
                    ty: ty.kind,
                    name: name,
                    offset: (self.temp_locals.len() + 1) * 8,
                };
                self.temp_locals.push(lvar);
                self.pos += 1;
            }
            func.paramnum = self.temp_locals.len();
            self.expect(")");
        }
        self.expect("{");

        while self.tokens[self.pos].op != "}" {
            func.body.push(self.stmt());
        }

        func.locals = self.temp_locals.clone();
        self.temp_locals = Vec::new();
        self.pos += 1;

        func
    }

    pub fn program(&mut self) {
        let mut funcs: Vec<Function> = vec![];

        while self.tokens[self.pos].kind != TokenKind::TkEof {
            funcs.push(self.function());
        }

        self.functions = funcs;
    }

    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            pos: 0,
            temp_locals: vec![],
            functions: vec![],
        }
    }

    fn expect(&mut self, op: &str) {
        if &self.tokens[self.pos].op != op {
            eprintln!("expected {} but got {}", op, self.tokens[self.pos].op);
            process::exit(1);
        }
        self.pos += 1;
    }

    fn expect_ident(&mut self) -> String {
        if self.tokens[self.pos].kind != TokenKind::TkIdent {
            eprintln!("expected identifier but got {}", self.tokens[self.pos].op);
            process::exit(1);
        }
        let ident = &self.tokens[self.pos].op;
        self.pos += 1;

        ident.to_string()
    }
}

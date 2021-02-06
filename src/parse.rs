use std::process;

use crate::tokenize::{Token, TokenKind};

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

pub struct LVar {
    pub name: String,
    pub offset: usize,
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
    pub nodes: Vec<Node>,
    pub locals: Vec<LVar>,
}

impl<'a> Parser<'a> {
    fn find_lvar(&mut self, name: String) -> usize {
        for local in &self.locals {
            if local.name == name {
                return local.offset;
            }
        }

        0
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
            let offset = self.find_lvar(name.to_string());
            if offset != 0 {
                self.pos += 1;
                return Node::new_node_lv(offset);
            }

            let offset = (self.locals.len() + 1) * 8;
            let lvar = LVar {
                name: name.to_string(),
                offset: offset,
            };

            self.locals.push(lvar);
            self.pos += 1;
            return Node::new_node_lv(offset);
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
        let node;
        if self.tokens[self.pos].op == "return" {
            self.pos += 1;
            node = Node::new_node_return(Box::new(self.expr()));
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

    pub fn program(&mut self) {
        let mut nodes: Vec<Node> = vec![];

        while self.tokens[self.pos].kind != TokenKind::TkEof {
            nodes.push(self.stmt());
        }

        self.nodes = nodes;
    }

    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            pos: 0,
            nodes: vec![],
            locals: vec![],
        }
    }
}
use std::process;

use crate::tokenize::{Token, TokenKind};
use crate::types::{Type, TypeKind};

#[derive(PartialEq)]
pub enum NodeKind {
    NdAdd,   // +
    NdSub,   // -
    NdMul,   // *
    NdDiv,   // /
    NdNum,   // number
    NdMt,    // more than >
    NdLt,    // less than <
    NdOm,    // or more >=
    NdOl,    // or less <=
    NdEq,    // equal
    NdNe,    // not equal
    NdAs,    // assign =
    NdLv,    // local variable
    NdIf,    // if
    NdWhile, // while
    NdFor,   // for
    NdBlock, // block {}
    NdFunc,  // function
    NdRt,    // return
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
    fn new_node(kind: NodeKind) -> Self {
        Self {
            kind: kind,
            ..Default::default()
        }
    }

    fn new_binary(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            lhs: Some(lhs),
            rhs: Some(rhs),
            ..Node::new_node(kind)
        }
    }

    fn new_unary(kind: NodeKind, expr: Box<Node>) -> Self {
        Self {
            lhs: Some(expr),
            ..Node::new_node(kind)
        }
    }

    fn new_node_lv(offset: usize) -> Self {
        Self {
            offset: offset,
            ..Node::new_node(NodeKind::NdLv)
        }
    }

    fn new_node_num(val: u32) -> Self {
        Self {
            val: val,
            ..Node::new_node(NodeKind::NdNum)
        }
    }
}

#[derive(Clone)]
pub struct LVar {
    pub ty: TypeKind,
    pub name: String,
    pub offset: usize,
}

impl LVar {
    fn new_lvar(ty: TypeKind, name: String, offset: usize) -> Self {
        Self {
            ty: ty,
            name: name,
            offset: offset,
        }
    }
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
        if self.consume(")") {
            return args;
        }

        args.push(self.add());
        while self.consume(",") {
            args.push(self.add());
        }
        self.expect(")");

        args
    }

    // primary = '(' expr ')' | ident ( "(" (args)* ")" )? | num
    fn primary(&mut self) -> Node {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")");
            return node;
        }

        if self.tokens[self.pos].kind == TokenKind::TkIdent {
            let name = &self.tokens[self.pos].op;

            self.pos += 1;
            if self.consume("(") {
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

    // unary = ( '+' | '-' )? primary
    fn unary(&mut self) -> Node {
        if self.consume("+") {
            return self.unary();
        }
        if self.consume("-") {
            return Node::new_binary(
                NodeKind::NdSub,
                Box::new(Node::new_node_num(0)),
                Box::new(self.unary()),
            );
        }

        self.primary()
    }

    // mul = unary ( '*' unary | '/' unary )*
    fn mul(&mut self) -> Node {
        let mut lhs = self.unary();

        loop {
            if self.consume("*") {
                lhs = Node::new_binary(NodeKind::NdMul, Box::new(lhs), Box::new(self.unary()));
            } else if self.consume("/") {
                lhs = Node::new_binary(NodeKind::NdDiv, Box::new(lhs), Box::new(self.unary()));
            } else {
                break;
            }
        }

        lhs
    }

    // add = mul ( "+" mul | "-" mul )*
    fn add(&mut self) -> Node {
        let mut lhs = self.mul();

        loop {
            if self.consume("+") {
                lhs = Node::new_binary(NodeKind::NdAdd, Box::new(lhs), Box::new(self.mul()));
            } else if self.consume("-") {
                lhs = Node::new_binary(NodeKind::NdSub, Box::new(lhs), Box::new(self.mul()));
            } else {
                break;
            }
        }

        lhs
    }

    // relational = add ( ">" add | "<" add | ">=" add | "<=" add )*
    fn relational(&mut self) -> Node {
        let mut lhs = self.add();

        loop {
            if self.consume(">") {
                lhs = Node::new_binary(NodeKind::NdMt, Box::new(lhs), Box::new(self.add()));
            } else if self.consume("<") {
                lhs = Node::new_binary(NodeKind::NdLt, Box::new(lhs), Box::new(self.add()));
            } else if self.consume(">=") {
                lhs = Node::new_binary(NodeKind::NdOm, Box::new(lhs), Box::new(self.add()));
            } else if self.consume("<=") {
                lhs = Node::new_binary(NodeKind::NdOl, Box::new(lhs), Box::new(self.add()));
            } else {
                break;
            }
        }

        lhs
    }

    // equality = relational ( "==" relational | "!=" relational )*
    fn equality(&mut self) -> Node {
        let mut lhs = self.relational();

        loop {
            if self.consume("==") {
                lhs = Node::new_binary(NodeKind::NdEq, Box::new(lhs), Box::new(self.mul()));
            } else if self.consume("!=") {
                lhs = Node::new_binary(NodeKind::NdNe, Box::new(lhs), Box::new(self.mul()));
            } else {
                break;
            }
        }

        lhs
    }

    // assign = equality ( "=" assign )?
    fn assign(&mut self) -> Node {
        let mut lhs = self.equality();

        if self.consume("=") {
            lhs = Node::new_binary(NodeKind::NdAs, Box::new(lhs), Box::new(self.assign()));
        }

        lhs
    }

    // expr = assign
    fn expr(&mut self) -> Node {
        return self.assign();
    }

    // stmt = "return" expr ";"
    //        | type ident ";"
    //        | "{" stmt* "}"
    //        | "if" "(" cond ")" stmt ( "else" stmt )?
    //        | "while" "(" cond ")" stmt
    //        | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //        | expr ";"
    fn stmt(&mut self) -> Node {
        let mut node;

        if self.consume("{") {
            node = Node::new_node(NodeKind::NdBlock);
            node.blocks = vec![];
            while !self.consume("}") {
                node.blocks.push(self.stmt());
            }
            return node;
        }

        let ty = Type::consume_type(&self.tokens[self.pos].op);
        if ty.kind != TypeKind::TyNone {
            self.pos += 1;
            let name = self.expect_ident();
            let offset = (self.temp_locals.len() + 1) * 8;
            let lvar = LVar::new_lvar(ty.kind, name, offset);
            self.expect(";");

            self.temp_locals.push(lvar);
            return Node::new_node_lv(offset);
        }

        if self.consume("return") {
            node = Node::new_unary(NodeKind::NdRt, Box::new(self.expr()));
        } else if self.consume("if") {
            node = Node::new_node(NodeKind::NdIf);
            self.expect("(");
            node.cond = Some(Box::new(self.expr()));
            self.expect(")");
            node.then = Some(Box::new(self.stmt()));
            if self.consume("else") {
                node.els = Some(Box::new(self.stmt()));
            }

            return node;
        } else if self.consume("while") {
            node = Node::new_node(NodeKind::NdWhile);
            self.expect("(");
            node.cond = Some(Box::new(self.expr()));
            self.expect(")");
            node.then = Some(Box::new(self.stmt()));

            return node;
        } else if self.consume("for") {
            node = Node::new_node(NodeKind::NdFor);
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

        self.expect(";");

        node
    }

    // function = type ident "(" (type params)* ")" "{" stmt* "}"
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
        if self.consume(")") {
        } else {
            let mut ty = Type::consume_type(&self.tokens[self.pos].op);
            self.pos += 1;
            let mut name = self.expect_ident();
            let lvar = LVar {
                ty: ty.kind,
                name: name,
                offset: (self.temp_locals.len() + 1) * 8,
            };
            self.temp_locals.push(lvar);

            while self.consume(",") {
                ty = Type::consume_type(&self.tokens[self.pos].op);
                self.pos += 1;
                name = self.expect_ident();
                let lvar = LVar {
                    ty: ty.kind,
                    name: name,
                    offset: (self.temp_locals.len() + 1) * 8,
                };
                self.temp_locals.push(lvar);
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

    // program = function*
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

    fn consume(&mut self, op: &str) -> bool {
        if &self.tokens[self.pos].op == op {
            self.pos += 1;
            return true;
        }

        false
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

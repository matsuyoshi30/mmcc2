use std::process;

use crate::tokenize::{Token, TokenKind};
use crate::types::{Type, TypeKind};

#[derive(PartialEq, Clone)]
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
    NdAddr,  // address &
    NdDeref, // dereference *
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

#[derive(Default, PartialEq, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub ty: Option<Box<Type>>,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: u32,
    pub lvar: Option<Box<LVar>>,
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

    fn new_node_lv(lvar: Box<LVar>) -> Self {
        Self {
            ty: Some(Box::new(lvar.ty.clone())),
            offset: lvar.offset,
            lvar: Some(lvar),
            ..Node::new_node(NodeKind::NdLv)
        }
    }

    fn new_node_num(val: u32) -> Self {
        Self {
            val: val,
            ..Node::new_node(NodeKind::NdNum)
        }
    }

    fn new_add(mut lhs: Box<Node>, mut rhs: Box<Node>) -> Self {
        lhs.check_type();
        rhs.check_type();

        if lhs.ty.as_ref().unwrap().is_integer() && rhs.ty.as_ref().unwrap().is_integer() {
            return Node::new_binary(NodeKind::NdAdd, lhs, rhs);
        }

        if lhs.kind == NodeKind::NdAddr {
            rhs = Box::new(Node::new_binary(
                NodeKind::NdMul,
                rhs,
                Box::new(Node::new_node_num(8)),
            ))
        }

        return Node::new_binary(NodeKind::NdSub, lhs, rhs);
    }

    fn new_sub(mut lhs: Box<Node>, mut rhs: Box<Node>) -> Self {
        lhs.check_type();
        rhs.check_type();

        if lhs.ty.as_ref().unwrap().is_integer() && rhs.ty.as_ref().unwrap().is_integer() {
            return Node::new_binary(NodeKind::NdSub, lhs, rhs);
        }

        if lhs.kind == NodeKind::NdAddr {
            rhs = Box::new(Node::new_binary(
                NodeKind::NdMul,
                rhs,
                Box::new(Node::new_node_num(8)),
            ))
        }

        return Node::new_binary(NodeKind::NdAdd, lhs, rhs);
    }

    fn check_type(&mut self) {
        if self.ty != None {
            return;
        }

        if let Some(n) = &mut self.lhs {
            n.check_type();
        }
        if let Some(n) = &mut self.rhs {
            n.check_type();
        }

        for n in &mut self.blocks {
            n.check_type();
        }
        for n in &mut self.args {
            n.check_type();
        }

        match self.kind {
            NodeKind::NdAdd
            | NodeKind::NdSub
            | NodeKind::NdMul
            | NodeKind::NdDiv
            | NodeKind::NdAs => {
                self.ty = self.lhs.clone().unwrap().ty;
                return;
            }
            NodeKind::NdLv => {
                self.ty = Some(Box::new(self.lvar.clone().unwrap().ty));
                return;
            }
            NodeKind::NdAddr => {
                self.ty = Some(Box::new(self.lhs.clone().unwrap().ty.unwrap().pointer_to()));
                return;
            }
            NodeKind::NdDeref => {
                self.ty = Some(self.lhs.clone().unwrap().ty.unwrap().ptr_to.unwrap());
                return;
            }
            NodeKind::NdFunc | NodeKind::NdNum => {
                self.ty = Some(Box::new(Type::new_int()));
                return;
            }
            _ => {}
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct LVar {
    pub ty: Type,
    pub name: String,
    pub offset: usize,
}

impl LVar {
    fn new_lvar(ty: Type, name: String, offset: usize) -> Self {
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
    fn find_lvar(&mut self, name: String) -> LVar {
        for local in &self.temp_locals {
            if local.name == name {
                return local.clone();
            }
        }

        eprintln!("Does not match any local variable");
        process::exit(1);
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
                let lvar = self.find_lvar(name.to_string());
                return Node::new_node_lv(Box::new(lvar));
            }
        }

        self.pos += 1;
        Node::new_node_num(self.tokens[self.pos - 1].val)
    }

    // unary = ( '+' | '-' )? primary | ('&' | '*') unary
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
        if self.consume("&") {
            return Node::new_unary(NodeKind::NdAddr, Box::new(self.unary()));
        }
        if self.consume("*") {
            return Node::new_unary(NodeKind::NdDeref, Box::new(self.unary()));
        }
        if self.consume("sizeof") {
            let mut node = self.unary();
            node.check_type();
            if node.ty.as_ref().unwrap().is_integer() {
                return Node::new_node_num(4);
            }
            return Node::new_node_num(8);
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
                lhs = Node::new_add(Box::new(lhs), Box::new(self.mul()));
            } else if self.consume("-") {
                lhs = Node::new_sub(Box::new(lhs), Box::new(self.mul()));
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
                let mut block = self.stmt();
                block.check_type();
                node.blocks.push(block);
            }
            return node;
        }

        let ty = self.consume_type();
        if ty.kind != TypeKind::TyNone {
            let name = self.expect_ident();
            let offset = (self.temp_locals.len() + 1) * 8;
            let lvar = LVar::new_lvar(ty, name, offset);
            self.expect(";");

            self.temp_locals.push(lvar.clone());
            let mut node = Node::new_node_lv(Box::new(lvar));
            node.check_type();
            return node;
        }

        if self.consume("return") {
            node = Node::new_unary(NodeKind::NdRt, Box::new(self.expr()));
        } else if self.consume("if") {
            node = Node::new_node(NodeKind::NdIf);
            self.expect("(");
            let mut cond = self.expr();
            cond.check_type();
            node.cond = Some(Box::new(cond));
            self.expect(")");
            let mut then = self.stmt();
            then.check_type();
            node.then = Some(Box::new(then));
            if self.consume("else") {
                let mut els = self.stmt();
                els.check_type();
                node.els = Some(Box::new(els));
            }

            return node;
        } else if self.consume("while") {
            node = Node::new_node(NodeKind::NdWhile);
            self.expect("(");
            let mut cond = self.expr();
            cond.check_type();
            node.cond = Some(Box::new(cond));
            self.expect(")");
            let mut then = self.stmt();
            then.check_type();
            node.then = Some(Box::new(then));

            return node;
        } else if self.consume("for") {
            node = Node::new_node(NodeKind::NdFor);
            self.expect("(");
            if self.tokens[self.pos].op != ";" {
                let mut preop = self.expr();
                preop.check_type();
                node.preop = Some(Box::new(preop));
            }
            self.expect(";");
            if self.tokens[self.pos].op != ";" {
                let mut cond = self.expr();
                cond.check_type();
                node.cond = Some(Box::new(cond));
            }
            self.expect(";");
            if self.tokens[self.pos].op != ")" {
                let mut postop = self.expr();
                postop.check_type();
                node.postop = Some(Box::new(postop));
            }
            self.expect(")");
            let mut then = self.stmt();
            then.check_type();
            node.then = Some(Box::new(then));

            return node;
        } else {
            node = self.expr();
        }

        self.expect(";");

        node
    }

    // function = type ident "(" (type params)* ")" "{" stmt* "}"
    fn function(&mut self) -> Function {
        let ty = self.consume_type();
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
            let mut ty = self.consume_type();
            let mut name = self.expect_ident();
            let lvar = LVar::new_lvar(ty, name, (self.temp_locals.len() + 1) * 8);
            self.temp_locals.push(lvar);

            while self.consume(",") {
                ty = self.consume_type();
                name = self.expect_ident();
                let lvar = LVar::new_lvar(ty, name, (self.temp_locals.len() + 1) * 8);
                self.temp_locals.push(lvar);
            }
            func.paramnum = self.temp_locals.len();
            self.expect(")");
        }
        self.expect("{");

        while self.tokens[self.pos].op != "}" {
            let mut node = self.stmt();
            node.check_type();
            func.body.push(node);
        }

        func.locals = self.temp_locals.clone();
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

    fn consume_type(&mut self) -> Type {
        let mut ty = Type {
            ..Default::default()
        };
        if self.consume("int") {
            ty = Type::new_int();
        }

        while self.consume("*") {
            ty = ty.pointer_to();
        }

        ty
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

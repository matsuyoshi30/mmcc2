use std::env;
use std::process;

fn strtol(s: &str) -> (&str, String) {
    let n = s.find(|c: char| !c.is_digit(10)).unwrap_or(s.len());
    let (op, r) = s.split_at(n);

    (op, r.to_string())
}

#[derive(PartialEq)]
enum TokenKind {
    TkReserved,
    TkIdent,
    TkNum,
    TkEof,
}

impl Default for TokenKind {
    fn default() -> Self {
        TokenKind::TkNum
    }
}

#[derive(Default)]
struct Token {
    kind: TokenKind,
    val: u32,
    op: String,
}

impl Token {
    fn new_token(kind: TokenKind, op: String) -> Self {
        Self {
            kind: kind,
            op: op,
            ..Default::default()
        }
    }

    fn new_token_num(val: u32) -> Self {
        Self {
            val: val,
            ..Default::default()
        }
    }
}

fn tokenize(s: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut expr = s;
    while let Some(c) = expr.chars().nth(0) {
        if c.is_whitespace() {
            expr = expr.split_off(1);
            continue;
        }

        if c == '>' || c == '<' {
            if expr.chars().nth(1).unwrap() == '=' {
                let v = expr.split_off(2);
                tokens.push(Token::new_token(TokenKind::TkReserved, expr));
                expr = v;
                continue;
            }
            tokens.push(Token::new_token(TokenKind::TkReserved, c.to_string()));
            expr = expr.split_off(1);
            continue;
        }

        if c == '=' || c == '!' {
            if expr.chars().nth(1).unwrap() == '=' {
                let v = expr.split_off(2);
                tokens.push(Token::new_token(TokenKind::TkReserved, expr));
                expr = v;
                continue;
            } else {
                if expr.chars().nth(1).unwrap() == '!' {
                    eprintln!("unable tokenize");
                    process::exit(1);
                } else {
                    tokens.push(Token::new_token(TokenKind::TkReserved, c.to_string()));
                    expr = expr.split_off(1);
                    continue;
                }
            }
        }

        if c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')' {
            tokens.push(Token::new_token(TokenKind::TkReserved, c.to_string()));
            expr = expr.split_off(1);
            continue;
        }

        if c.is_ascii_alphabetic() {
            tokens.push(Token::new_token(TokenKind::TkIdent, c.to_string()));
            expr = expr.split_off(1);
            continue;
        }

        if c.is_digit(10) {
            let (n, r) = strtol(&expr);
            let token = Token::new_token_num(n.parse().unwrap());
            tokens.push(token);
            expr = r;
            continue;
        }

        if c == ';' {
            tokens.push(Token::new_token(TokenKind::TkReserved, c.to_string()));
            expr = expr.split_off(1);
            continue;
        }
    }
    tokens.push(Token::new_token(TokenKind::TkEof, " ".to_string()));

    tokens
}

#[derive(PartialEq)]
enum NodeKind {
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
}

impl Default for NodeKind {
    fn default() -> Self {
        NodeKind::NdNum
    }
}

#[derive(Default)]
struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: u32,
    offset: u8,
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

    fn new_node_lv(offset: u8) -> Self {
        Self {
            kind: NodeKind::NdLv,
            offset: offset,
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

struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
    nodes: Vec<Node>,
}

impl<'a> Parser<'a> {
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
            let c = self.tokens[self.pos].op.chars().nth(0).unwrap() as u8;
            let offset = (c - 97 + 1) * 8; // (c - 'a' + 1) * 8
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
        let node = self.expr();

        if self.tokens[self.pos].op.as_str() != ";" {
            eprintln!("expected ';'");
            process::exit(1)
        }
        self.pos += 1;

        node
    }

    fn program(&mut self) {
        let mut nodes: Vec<Node> = vec![];

        while self.tokens[self.pos].kind != TokenKind::TkEof {
            nodes.push(self.stmt());
        }

        self.nodes = nodes;
    }

    fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            pos: 0,
            nodes: vec![],
        }
    }
}

fn gen_lval(node: Box<Node>) {
    if node.kind != NodeKind::NdLv {
        eprintln!("The left value of the assignment is not a variable.");
        process::exit(1);
    }

    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset);
    println!("  push rax");
}

fn gen(node: Box<Node>) {
    if node.kind == NodeKind::NdNum {
        println!("  push {}", node.val);
        return;
    }

    if node.kind == NodeKind::NdLv {
        gen_lval(node);
        println!("  pop rax");
        println!("  mov rax, [rax]"); // load value from the address in rax into rax
        println!("  push rax");
        return;
    }

    if node.kind == NodeKind::NdAs {
        gen_lval(node.lhs.unwrap());
        gen(node.rhs.unwrap());

        println!("  pop rdi");
        println!("  pop rax");
        println!("  mov [rax], rdi"); // store value from rdi into the address in rax
        println!("  push rdi");
        return;
    }

    gen(node.lhs.unwrap());
    gen(node.rhs.unwrap());

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::NdAdd => {
            println!("  add rax, rdi");
        }
        NodeKind::NdSub => {
            println!("  sub rax, rdi");
        }
        NodeKind::NdMul => {
            println!("  imul rax, rdi");
        }
        NodeKind::NdDiv => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::NdMt => {
            println!("  cmp rdi, rax");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::NdLt => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::NdOm => {
            println!("  cmp rdi, rax");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        NodeKind::NdOl => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        NodeKind::NdEq => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::NdNe => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        _ => {
            eprintln!("unknown node");
            process::exit(1);
        }
    }

    println!("  push rax");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("wrong the number of arguments");
        process::exit(1);
    }

    let input = &args[1];
    let tokens = tokenize(input.to_string());

    let mut parser = Parser::new(&tokens);
    parser.program();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // prologue
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", 26 * 8);

    for node in parser.nodes {
        gen(Box::new(node));
        println!("  pop rax");
    }

    // epilogue
    println!("  mov rsp, rbp");
    println!("  pop rbp");

    println!("  ret");
}

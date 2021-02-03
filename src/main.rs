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
    TkNum,
    TkEof,
}

struct Token {
    kind: TokenKind,
    val: u32,
    op: char,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            kind: TokenKind::TkEof,
            val: 0,
            op: ' ',
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
            let token = Token {
                kind: TokenKind::TkReserved,
                op: c,
                ..Default::default()
            };
            tokens.push(token);
            expr = expr.split_off(1);
            continue;
        }

        if c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')' {
            let token = Token {
                kind: TokenKind::TkReserved,
                op: c,
                ..Default::default()
            };
            tokens.push(token);
            expr = expr.split_off(1);
            continue;
        }

        if c.is_digit(10) {
            let (n, r) = strtol(&expr);
            let token = Token {
                kind: TokenKind::TkNum,
                val: n.parse().unwrap(),
                ..Default::default()
            };
            tokens.push(token);
            expr = r;
            continue;
        }
    }
    tokens.push(Default::default());

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
}

struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn primary(&mut self) -> Node {
        if self.tokens[self.pos].op == '(' {
            self.pos += 1;
            let node = self.expr();
            if self.tokens[self.pos].op != ')' {
                eprintln!("unexpected token: {}", self.tokens[self.pos].op);
                process::exit(1);
            }
            self.pos += 1;
            return node;
        }

        self.pos += 1;
        Node {
            val: self.tokens[self.pos - 1].val,
            ..Default::default()
        }
    }

    fn unary(&mut self) -> Node {
        if self.tokens[self.pos].op == '+' {
            self.pos += 1;
            return self.unary();
        }
        if self.tokens[self.pos].op == '-' {
            self.pos += 1;
            let rhs = self.unary();
            return Node {
                kind: NodeKind::NdSub,
                lhs: Some(Box::new(Node {
                    val: 0,
                    ..Default::default()
                })),
                rhs: Some(Box::new(rhs)),
                ..Default::default()
            };
        }

        self.primary()
    }

    fn mul(&mut self) -> Node {
        let mut lhs = self.unary();

        loop {
            match self.tokens[self.pos].op {
                '*' => {
                    self.pos += 1;
                    let rhs = self.unary();
                    lhs = Node {
                        kind: NodeKind::NdMul,
                        lhs: Some(Box::new(lhs)),
                        rhs: Some(Box::new(rhs)),
                        ..Default::default()
                    }
                }
                '/' => {
                    self.pos += 1;
                    let rhs = self.unary();
                    lhs = Node {
                        kind: NodeKind::NdDiv,
                        lhs: Some(Box::new(lhs)),
                        rhs: Some(Box::new(rhs)),
                        ..Default::default()
                    };
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
            match self.tokens[self.pos].op {
                '+' => {
                    self.pos += 1;
                    let rhs = self.mul();
                    lhs = Node {
                        kind: NodeKind::NdAdd,
                        lhs: Some(Box::new(lhs)),
                        rhs: Some(Box::new(rhs)),
                        ..Default::default()
                    };
                }
                '-' => {
                    self.pos += 1;
                    let rhs = self.mul();
                    lhs = Node {
                        kind: NodeKind::NdSub,
                        lhs: Some(Box::new(lhs)),
                        rhs: Some(Box::new(rhs)),
                        ..Default::default()
                    };
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
            match self.tokens[self.pos].op {
                '>' => {
                    self.pos += 1;
                    let rhs = self.add();
                    lhs = Node {
                        kind: NodeKind::NdMt,
                        lhs: Some(Box::new(lhs)),
                        rhs: Some(Box::new(rhs)),
                        ..Default::default()
                    };
                }
                '<' => {
                    self.pos += 1;
                    let rhs = self.add();
                    lhs = Node {
                        kind: NodeKind::NdLt,
                        lhs: Some(Box::new(lhs)),
                        rhs: Some(Box::new(rhs)),
                        ..Default::default()
                    };
                }
                _ => {
                    break;
                }
            }
        }

        lhs
    }

    fn expr(&mut self) -> Node {
        return self.relational();
    }

    fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            pos: 0,
        }
    }
}

fn gen(node: Box<Node>) {
    if node.kind == NodeKind::NdNum {
        println!("  push {}", node.val);
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
            println!("  setle al");
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
    let node = parser.expr();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen(Box::new(node));

    println!("  pop rax");
    println!("  ret");
}

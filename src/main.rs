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

fn tokenize(s: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut expr = s;
    while let Some(c) = expr.chars().nth(0) {
        if c.is_whitespace() {
            expr = expr.split_off(1);
            continue;
        }

        if c == '+' || c == '-' {
            let token = Token {
                kind: TokenKind::TkReserved,
                op: c,
                val: 0,
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
                op: ' ',
            };
            tokens.push(token);
            expr = r;
            continue;
        }
    }
    tokens.push(Token {
        kind: TokenKind::TkEof,
        val: 0,
        op: ' ',
    });

    tokens
}

#[derive(PartialEq)]
enum NodeKind {
    NdAdd,
    NdSub,
    NdNum,
}

struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: u32,
}

fn expr(tokens: Vec<Token>) -> Node {
    let mut pos = 0;
    let mut lhs = Node {
        kind: NodeKind::NdNum,
        lhs: None,
        rhs: None,
        val: tokens[pos].val,
    };

    pos += 1;
    loop {
        match tokens[pos].op {
            '+' => {
                pos += 1;
                lhs = Node {
                    kind: NodeKind::NdAdd,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(Node {
                        kind: NodeKind::NdNum,
                        lhs: None,
                        rhs: None,
                        val: tokens[pos].val,
                    })),
                    val: 0,
                };
                pos += 1;
            }
            '-' => {
                pos += 1;
                lhs = Node {
                    kind: NodeKind::NdSub,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(Node {
                        kind: NodeKind::NdNum,
                        lhs: None,
                        rhs: None,
                        val: tokens[pos].val,
                    })),
                    val: 0,
                };
                pos += 1;
            }
            _ => {
                break;
            }
        }
    }

    lhs
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
    let node = expr(tokens);

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen(Box::new(node));

    println!("  pop rax");
    println!("  ret");
}

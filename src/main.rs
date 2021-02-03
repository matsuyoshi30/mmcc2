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

fn primary(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    if tokens[pos].op == '(' {
        pos += 1;
        let (node, npos) = expr(&tokens, pos);
        pos = npos;
        if tokens[pos].op != ')' {
            eprintln!("unexpected token: {}", tokens[pos].op);
            process::exit(1);
        }
        pos += 1;
        return (node, pos);
    }

    pos += 1;
    (
        Node {
            val: tokens[pos - 1].val,
            ..Node::default()
        },
        pos,
    )
}

fn unary(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    if tokens[pos].op == '+' {
        pos += 1;
        return unary(&tokens, pos);
    }
    if tokens[pos].op == '-' {
        pos += 1;
        let (rhs, npos) = unary(&tokens, pos);
        return (
            Node {
                kind: NodeKind::NdSub,
                lhs: Some(Box::new(Node {
                    val: 0,
                    ..Default::default()
                })),
                rhs: Some(Box::new(rhs)),
                ..Default::default()
            },
            npos,
        );
    }

    primary(&tokens, pos)
}

fn mul(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    let (mut lhs, npos) = unary(&tokens, pos);

    pos = npos;
    loop {
        match tokens[pos].op {
            '*' => {
                pos += 1;
                let (rhs, npos) = unary(&tokens, pos);
                lhs = Node {
                    kind: NodeKind::NdMul,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(rhs)),
                    ..Default::default()
                };
                pos = npos;
            }
            '/' => {
                pos += 1;
                let (rhs, npos) = unary(&tokens, pos);
                lhs = Node {
                    kind: NodeKind::NdDiv,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(rhs)),
                    ..Default::default()
                };
                pos = npos;
            }
            _ => {
                break;
            }
        }
    }

    (lhs, pos)
}

fn add(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    let (mut lhs, npos) = mul(tokens, pos);

    pos = npos;
    loop {
        match tokens[pos].op {
            '+' => {
                pos += 1;
                let (rhs, npos) = mul(&tokens, pos);
                lhs = Node {
                    kind: NodeKind::NdAdd,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(rhs)),
                    ..Default::default()
                };
                pos = npos;
            }
            '-' => {
                pos += 1;
                let (rhs, npos) = mul(&tokens, pos);
                lhs = Node {
                    kind: NodeKind::NdSub,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(rhs)),
                    ..Default::default()
                };
                pos = npos;
            }
            _ => {
                break;
            }
        }
    }

    (lhs, pos)
}

fn relational(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    let (mut lhs, npos) = add(tokens, pos);

    pos = npos;
    loop {
        match tokens[pos].op {
            '>' => {
                pos += 1;
                let (rhs, npos) = add(&tokens, pos);
                lhs = Node {
                    kind: NodeKind::NdMt,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(rhs)),
                    ..Default::default()
                };
                pos = npos;
            }
            '<' => {
                pos += 1;
                let (rhs, npos) = add(&tokens, pos);
                lhs = Node {
                    kind: NodeKind::NdLt,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(rhs)),
                    ..Default::default()
                };
                pos = npos;
            }
            _ => {
                break;
            }
        }
    }

    (lhs, pos)
}

fn expr(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    return relational(&tokens, pos);
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
    let (node, _) = expr(&tokens, 0);

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen(Box::new(node));

    println!("  pop rax");
    println!("  ret");
}

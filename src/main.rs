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
    op: String,
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
                op: c.to_string(),
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
                op: "".to_string(),
            };
            tokens.push(token);
            expr = r;
            continue;
        }
    }
    tokens.push(Token {
        kind: TokenKind::TkEof,
        val: 0,
        op: "".to_string(),
    });

    tokens
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("wrong the number of arguments");
        process::exit(1);
    }

    let input = &args[1];
    let mut tokens = tokenize(input.to_string());

    if tokens[0].kind != TokenKind::TkNum {
        eprintln!("input should be started as number");
        process::exit(1);
    }

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    println!("  mov rax, {}", tokens[0].val);
    tokens = tokens.split_off(1);

    let mut i = 0;
    while i != tokens.len() {
        match tokens[i] {
            Token {
                kind: TokenKind::TkReserved,
                ..
            } => {
                if tokens[i].op == '+'.to_string() {
                    println!("  add rax, {}", tokens[i + 1].val);
                } else {
                    println!("  sub rax, {}", tokens[i + 1].val);
                }
                i += 2;
                continue;
            }
            Token {
                kind: TokenKind::TkEof,
                ..
            } => break,
            _ => {
                eprintln!("unexpected character: {}", tokens[i].op);
                process::exit(1);
            }
        }
    }

    println!("  ret");
}

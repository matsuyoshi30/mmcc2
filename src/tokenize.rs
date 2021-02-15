use std::process;

fn strtol(s: &str) -> (&str, String) {
    let n = s.find(|c: char| !c.is_digit(10)).unwrap_or(s.len());
    let (op, r) = s.split_at(n);

    (op, r.to_string())
}

fn strtos(s: &str) -> (String, String) {
    let n = s.find(|c: char| !is_alnum(c)).unwrap_or(s.len());
    let (op, r) = s.split_at(n);

    (op.to_string(), r.to_string())
}

fn strchr(s: &str, ch: char) -> bool {
    let vec = s.chars().collect::<Vec<char>>();
    vec.contains(&ch)
}

fn is_alnum(c: char) -> bool {
    c.is_ascii_alphabetic() || c.is_digit(10) || c == '_'
}

#[derive(PartialEq)]
pub enum TokenKind {
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
pub struct Token {
    pub kind: TokenKind,
    pub val: u32,
    pub op: String,
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

fn is_reserved(s: &str) -> bool {
    let keywords = ["return", "if", "else", "while", "for", "sizeof"];
    for keyword in &keywords {
        if &s == keyword {
            return true;
        }
    }

    let types = ["int"];
    for ty in &types {
        if &s == ty {
            return true;
        }
    }

    false
}

pub fn tokenize(s: String) -> Vec<Token> {
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

        if strchr("+-*/(){}[],;&", c) {
            tokens.push(Token::new_token(TokenKind::TkReserved, c.to_string()));
            expr = expr.split_off(1);
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let (s, r) = strtos(&expr);
            if is_reserved(&s) {
                tokens.push(Token::new_token(TokenKind::TkReserved, s));
            } else {
                tokens.push(Token::new_token(TokenKind::TkIdent, s));
            }
            expr = r;
            continue;
        }

        if c.is_digit(10) {
            let (n, r) = strtol(&expr);
            let token = Token::new_token_num(n.parse().unwrap());
            tokens.push(token);
            expr = r;
            continue;
        }
    }
    tokens.push(Token::new_token(TokenKind::TkEof, " ".to_string()));

    tokens
}

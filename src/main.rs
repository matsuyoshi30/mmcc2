use std::env;
use std::process;

fn strtol(s: &str) -> (&str, String) {
    let n = s.find(|c: char| !c.is_digit(10)).unwrap_or(s.len());
    let (op, r) = s.split_at(n);

    (op, r.to_string())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("wrong the number of arguments");
        process::exit(1);
    }

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    let (n, mut expr) = strtol(&args[1]);
    println!("  mov rax, {}", n);

    while let Some(c) = expr.chars().nth(0) {
        let r = expr.split_off(1);

        let (n, p) = strtol(&r);
        expr = p;

        match c {
            '+' => {
                println!("  add rax, {}", n);
                continue;
            }
            '-' => {
                println!("  sub rax, {}", n);
                continue;
            }
            _ => {
                eprintln!("unexpected character: {}", c);
                process::exit(1);
            }
        }
    }

    println!("  ret");
}

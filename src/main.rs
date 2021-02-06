use std::env;
use std::process;

mod codegen;
mod parse;
mod tokenize;

use crate::codegen::Generator;
use crate::parse::Parser;
use crate::tokenize::tokenize;

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
    println!("  sub rsp, {}", (parser.locals.len() + 1) * 8);

    let mut generator = Generator::new();
    for node in parser.nodes {
        generator.gen(Box::new(node));
        println!("  pop rax");
    }

    // epilogue
    println!("  mov rsp, rbp");
    println!("  pop rbp");

    println!("  ret");
}

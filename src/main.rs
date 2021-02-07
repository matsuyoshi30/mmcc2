use std::env;
use std::process;

mod codegen;
mod parse;
mod tokenize;

use crate::codegen::Generator;
use crate::parse::Parser;
use crate::tokenize::tokenize;

fn align(mut n: usize, align: usize) -> usize {
    if n < align {
        return align;
    }
    while n % align == 0 {
        n += 1;
    }
    return n;
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
    for function in parser.functions {
        println!(".global {}", function.name);
        println!("{}:", function.name);

        // prologue
        println!("  push rbp");
        println!("  mov rbp, rsp");
        println!("  sub rsp, {}", align((function.locals.len() + 1) * 8, 16));

        let mut generator = Generator::new();
        for node in function.body {
            generator.gen(Box::new(node));
            println!("  pop rax");
        }

        // epilogue
        println!("  mov rsp, rbp");
        println!("  pop rbp");

        println!("  ret");
    }
}

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

    let mut generator = Generator::new();
    generator.codegen(parser);
}

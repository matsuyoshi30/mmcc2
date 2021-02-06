use std::process;

use crate::parse::{Node, NodeKind};

fn gen_lval(node: Box<Node>) {
    if node.kind != NodeKind::NdLv {
        eprintln!("The left value of the assignment is not a variable.");
        process::exit(1);
    }

    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset);
    println!("  push rax");
}

pub fn gen(node: Box<Node>) {
    if node.kind == NodeKind::NdRt {
        gen(node.lhs.unwrap());
        println!("  pop rax");
        println!("  mov rsp, rbp");
        println!("  pop rbp");
        println!("  ret");
        return;
    }

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

    match node.lhs {
        Some(inner) => gen(inner),
        _ => (),
    }
    match node.rhs {
        Some(inner) => gen(inner),
        _ => (),
    }

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

use std::process;

use crate::parse::{Node, NodeKind};

pub struct Generator {
    label: u32,
}

impl Generator {
    fn new_label(&mut self) -> u32 {
        let label = self.label;
        self.label += 1;
        label
    }

    fn gen_lval(&mut self, node: Box<Node>) {
        if node.kind != NodeKind::NdLv {
            eprintln!("The left value of the assignment is not a variable.");
            process::exit(1);
        }

        println!("  mov rax, rbp");
        println!("  sub rax, {}", node.offset);
        println!("  push rax");
    }

    pub fn gen(&mut self, node: Box<Node>) {
        if node.kind == NodeKind::NdRt {
            self.gen(node.lhs.unwrap());
            println!("  pop rax");
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            return;
        }

        if node.kind == NodeKind::NdIf {
            let label = self.new_label();
            self.gen(node.cond.unwrap());
            println!("  pop rax");
            println!("  cmp rax, 0");
            match node.els {
                Some(els) => {
                    println!("  je .L.else.{}", label);
                    self.gen(node.then.unwrap());
                    println!("  jmp .L.end.{}", label);
                    println!(".L.else.{}:", label);
                    self.gen(els);
                    println!(".L.end.{}:", label);
                }
                None => {
                    println!("  je .L.end.{}", label);
                    self.gen(node.then.unwrap());
                    println!(".L.end.{}:", label);
                }
            }
            return;
        }

        if node.kind == NodeKind::NdNum {
            println!("  push {}", node.val);
            return;
        }

        if node.kind == NodeKind::NdLv {
            self.gen_lval(node);
            println!("  pop rax");
            println!("  mov rax, [rax]"); // load value from the address in rax into rax
            println!("  push rax");
            return;
        }

        if node.kind == NodeKind::NdAs {
            self.gen_lval(node.lhs.unwrap());
            self.gen(node.rhs.unwrap());

            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov [rax], rdi"); // store value from rdi into the address in rax
            println!("  push rdi");
            return;
        }

        match node.lhs {
            Some(inner) => self.gen(inner),
            _ => (),
        }
        match node.rhs {
            Some(inner) => self.gen(inner),
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

    pub fn new() -> Self {
        Self { label: 0 }
    }
}

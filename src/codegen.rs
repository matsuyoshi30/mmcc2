use std::process;

use crate::parse::{Node, NodeKind, Parser};
use crate::types::TypeKind;

pub struct Generator {
    label: u32,
    var_offsets: Vec<usize>,
}

static ARG_REGS4: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
static ARG_REGS8: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl Generator {
    fn new_label(&mut self) -> u32 {
        let label = self.label;
        self.label += 1;
        label
    }

    fn gen_lval(&mut self, node: Box<Node>) {
        if node.kind == NodeKind::NdLv {
            println!("  mov rax, rbp");
            println!("  sub rax, {}", self.var_offsets[node.lvar.unwrap().id]);
            println!("  push rax");
            return;
        }
        if node.kind == NodeKind::NdDeref {
            self.gen(node.lhs.unwrap());
            return;
        }

        eprintln!("The left value of the assignment is not a variable.");
        process::exit(1);
    }

    fn gen(&mut self, node: Box<Node>) {
        match node.kind {
            NodeKind::NdRt => {
                self.gen(node.lhs.unwrap());
                println!("  pop rax");
                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
                return;
            }
            NodeKind::NdBlock => {
                for block in node.blocks {
                    self.gen(Box::new(block));
                }
                return;
            }
            NodeKind::NdIf => {
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
            NodeKind::NdWhile => {
                let label = self.new_label();
                println!(".L.begin.{}:", label);
                self.gen(node.cond.unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .L.end.{}", label);
                self.gen(node.then.unwrap());
                println!("  jmp .L.begin.{}", label);
                println!(".L.end.{}:", label);
                return;
            }
            NodeKind::NdFor => {
                let label = self.new_label();
                if let Some(preop) = node.preop {
                    self.gen(preop);
                }
                println!(".L.begin.{}:", label);
                if let Some(cond) = node.cond {
                    self.gen(cond);
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .L.end.{}", label);
                }
                self.gen(node.then.unwrap());
                if let Some(postop) = node.postop {
                    self.gen(postop);
                }
                println!("  jmp .L.begin.{}", label);
                println!(".L.end.{}:", label);
                return;
            }
            NodeKind::NdFunc => {
                let len = node.args.len();
                for args in node.args {
                    self.gen(Box::new(args));
                }

                for n in (0..len).rev() {
                    println!("  pop {}", ARG_REGS8[n]);
                }

                println!("  call {}", node.funcname);
                println!("  push rax");
                return;
            }
            NodeKind::NdNum => {
                println!("  push {}", node.val);
                return;
            }
            NodeKind::NdLv => {
                let size = node.ty.as_ref().unwrap().size;
                let ty = node.ty.clone().unwrap().kind;

                self.gen_lval(node);
                if ty != TypeKind::TyArr {
                    println!("  pop rax");
                    if size == 4 {
                        println!("  movsxd rax, dword ptr [rax]");
                    } else {
                        println!("  mov rax, [rax]");
                    }
                    println!("  push rax");
                }
                return;
            }
            NodeKind::NdAs => {
                self.gen_lval(node.lhs.unwrap());
                self.gen(node.rhs.unwrap());

                println!("  pop rdi");
                println!("  pop rax");
                if node.ty.unwrap().size == 4 {
                    println!("  mov [rax], edi");
                } else {
                    println!("  mov [rax], rdi");
                }
                println!("  push rdi");
                return;
            }
            NodeKind::NdAddr => {
                self.gen_lval(node.lhs.unwrap());
                return;
            }
            NodeKind::NdDeref => {
                self.gen(node.lhs.unwrap());
                if node.ty.clone().unwrap().kind != TypeKind::TyArr {
                    println!("  pop rax");
                    if node.ty.unwrap().size == 4 {
                        println!("  movsxd rax, dword ptr [rax]");
                    } else {
                        println!("  mov rax, [rax]");
                    }
                    println!("  push rax");
                }
                return;
            }
            _ => {}
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

    pub fn codegen(&mut self, parser: Parser) {
        println!(".intel_syntax noprefix");
        for function in parser.functions {
            println!(".global {}", function.name);
            println!("{}:", function.name);

            self.var_offsets = vec![0; function.locals.len()];
            let mut stack_size = 0;
            for i in (0..function.locals.len()).rev() {
                println!("# ----- {}", function.locals[i].name);
                stack_size += function.locals[i].ty.size;
                self.var_offsets[i] = stack_size;
            }

            // prologue
            println!("  push rbp");
            println!("  mov rbp, rsp");
            println!("  sub rsp, {}", align(stack_size, 16));

            for n in 0..function.paramnum {
                if function.locals[n].ty.size == 4 {
                    println!("  mov [rbp-{}], {}", self.var_offsets[n], ARG_REGS4[n]);
                } else {
                    println!("  mov [rbp-{}], {}", self.var_offsets[n], ARG_REGS8[n]);
                }
            }

            for node in function.body {
                self.gen(Box::new(node));
                println!("  pop rax");
            }

            // epilogue
            println!("  mov rsp, rbp");
            println!("  pop rbp");

            println!("  ret");
        }
    }

    pub fn new() -> Self {
        Self {
            label: 0,
            var_offsets: vec![],
        }
    }
}

fn align(mut n: usize, align: usize) -> usize {
    if n < align {
        return align;
    }
    while n % align != 0 {
        n += 1;
    }
    return n;
}

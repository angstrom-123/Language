use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::process::exit;
use crate::definitions::TokenType;
use crate::lexer::Lexer;
use crate::parser::ExprType;
use crate::parser::ParseNode;

pub mod definitions;
pub mod lexer;
pub mod parser;

fn generate_nasm_x86(out_path: String, nodes: Vec<ParseNode>) -> std::io::Result<()> {
    let mut f = fs::File::create(out_path)?;
    writeln!(f, "; --- Header ---")?;
    writeln!(f, "global _start")?;
    writeln!(f, "section .text")?;
    writeln!(f, "; --- Debug Dump ---")?;
    writeln!(f, "dump:")?;
    writeln!(f, "    sub rsp, 40")?;
    writeln!(f, "    lea rsi, [rsp + 31]")?;
    writeln!(f, "    mov byte [rsp + 31], 10")?;
    writeln!(f, "    mov ecx, 1")?;
    writeln!(f, "    mov r8, -3689348814741910323")?;
    writeln!(f, ".LBB0_1:")?;
    writeln!(f, "    mov rax, rdi")?;
    writeln!(f, "    mul r8")?;
    writeln!(f, "    shr rdx, 3")?;
    writeln!(f, "    lea eax, [rdx + rdx]")?;
    writeln!(f, "    lea eax, [rax + 4*rax]")?;
    writeln!(f, "    mov r9d, edi")?;
    writeln!(f, "    sub r9d, eax")?;
    writeln!(f, "    or r9b, 48")?;
    writeln!(f, "    mov byte [rsi - 1], r9b")?;
    writeln!(f, "    dec rsi")?;
    writeln!(f, "    inc rcx")?;
    writeln!(f, "    cmp rdi, 9")?;
    writeln!(f, "    mov rdi, rdx")?;
    writeln!(f, "    ja .LBB0_1")?;
    writeln!(f, "    mov edi, 1")?;
    writeln!(f, "    mov rdx, rcx")?;
    writeln!(f, "    mov rax, 1")?;
    writeln!(f, "    syscall")?;
    writeln!(f, "    add rsp, 40")?;
    writeln!(f, "    ret")?;
    writeln!(f, "_start:")?;
    for node in nodes {
        match node.kind {
            ExprType::Program => {}, // Don't need to do anything for this node
            ExprType::Literal => {
                writeln!(f, "; --- Literal {} ---", node.tok.val_str())?;
                writeln!(f, "    mov rax, {}", node.tok.val_str())?;
                writeln!(f, "    push rax")?;
            },
            ExprType::Return => {
                writeln!(f, "; --- Return ---")?;
                writeln!(f, "    pop rdi")?;
                writeln!(f, "    mov rax, 60")?;
                writeln!(f, "    syscall")?;
            },
            ExprType::DebugDump => {
                writeln!(f, "; --- DebugDump ---")?;
                writeln!(f, "    pop rdi")?;
                writeln!(f, "    call dump")?;
            },
            ExprType::BinOp => {
                match node.tok.kind {
                    TokenType::OpPlus => {
                        writeln!(f, "; --- BinOp::OpPlus ---")?;
                        writeln!(f, "    pop rax")?;
                        writeln!(f, "    pop rbx")?;
                        writeln!(f, "    add rax, rbx")?;
                        writeln!(f, "    push rax")?;
                    },
                    TokenType::OpMinus => {
                        writeln!(f, "; --- BinOp::OpMinus---")?;
                        writeln!(f, "    pop rax")?;
                        writeln!(f, "    pop rbx")?;
                        writeln!(f, "    sub rbx, rax")?;
                        writeln!(f, "    push rbx")?;
                    },
                    TokenType::OpMul => {
                        writeln!(f, "; --- BinOp::OpMul ---")?;
                        writeln!(f, "    pop rax")?;
                        writeln!(f, "    pop rbx")?;
                        writeln!(f, "    imul rax, rbx")?;
                        writeln!(f, "    push rax")?;
                    },
                    TokenType::OpDiv => {
                        writeln!(f, "; --- BinOp::OpDiv ---")?;
                        writeln!(f, "    pop rcx")?;
                        writeln!(f, "    pop rax")?;
                        writeln!(f, "    xor rdx, rdx")?;
                        writeln!(f, "    idiv rcx")?;
                        writeln!(f, "    push rax")?;
                    },
                    _ => unimplemented!("Generating assembly for other bin ops"),
                }
            },
            _ => {
                unimplemented!("Parsing all tokens");
            },
        }
    }

    writeln!(f, "; --- Footer ---")?;
    writeln!(f, "    mov rdi, 0")?;
    writeln!(f, "    mov rax, 60")?;
    writeln!(f, "    syscall")?;

    Ok(())
}

pub fn compile(src_path: String, src_code: String) {
    eprintln!("Tokenizing:");
    let mut lexer: Lexer = Lexer::new(src_code);
    lexer.tokenize();
    for tok in &lexer.toks {
        eprintln!("[{:3}:{:3}]: {:.<14}", tok.pos.row, tok.pos.col, tok.val_str());
    }

    eprintln!("\nLexing:");
    lexer.lex();
    for tok in &lexer.toks {
        eprintln!("[{:3}:{:3}]: {:.<14} {:?}", tok.pos.row, tok.pos.col, tok.val_str(), tok.kind);
    }

    eprintln!("\nParsing:");
    let ast = &mut parser::ParseTree::new(src_path.clone());
    ast.construct(&mut lexer);
    ast.dump();

    eprintln!("\nGenerating asm:");
    let mut nodes: Vec<ParseNode> = Vec::new();
    ast.traverse(&mut nodes);
    
    let generate = generate_nasm_x86("output.asm".to_string(), nodes);
    let _ = generate.inspect_err(|e| panic!("Error: Failed to generate assembly: {e}"));
    eprintln!("{} -> output.asm", src_path.clone());

    eprintln!("\nAssembling:");
    let assemble = Command::new("nasm").arg("-f").arg("elf64").arg("-o").arg("output.o").arg("output.asm").output();
    let _ = assemble.inspect_err(|e| panic!("Error: Failed to assemble program: {e}"));
    eprintln!("output.asm -> output.o");

    eprintln!("\nLinking:");
    let link = Command::new("ld").arg("-o").arg("output").arg("output.o").output();
    let _ = link.inspect_err(|e| panic!("Error: Failed to link program: {e}"));
    eprintln!("output.o -> output");
    
    eprintln!("\nCompilation Complete");
}

pub fn simulate(src_path: String, src_code: String) {
    eprintln!("Tokenizing:");
    let mut lexer: Lexer = Lexer::new(src_code);
    lexer.tokenize();
    for tok in &lexer.toks {
        eprintln!("[{:3}:{:3}]: {:.<14}", tok.pos.row, tok.pos.col, tok.val_str());
    }

    eprintln!("\nLexing:");
    lexer.lex();
    for tok in &lexer.toks {
        eprintln!("[{:3}:{:3}]: {:.<14} {:?}", tok.pos.row, tok.pos.col, tok.val_str(), tok.kind);
    }

    eprintln!("\nParsing:");
    let ast = &mut parser::ParseTree::new(src_path);
    ast.construct(&mut lexer);
    ast.dump();

    eprintln!("\nSimulating:");
    let mut nodes: Vec<ParseNode> = Vec::new();
    ast.traverse(&mut nodes);

    let mut stack: Vec<i64> = Vec::new();
    for node in &nodes {
        match node.kind {
            ExprType::Program => {}, // Don't need to do anything for this node
            ExprType::Literal => {
                let val: i64 = node.tok.val_str().parse().expect("Error: Failed to parse string as int");
                stack.push(val);
            },
            ExprType::DebugDump => {
                let val: i64 = stack.pop().expect("Error: Failed to pop stack");
                println!("{}", val);
            },
            ExprType::Return => {
                let val: i64 = stack.pop().expect("Error: Failed to pop stack");
                exit(val.try_into().expect("Error: Failed to convert i64 to i32"));
            },
            ExprType::BinOp => {
                match node.tok.kind {
                    TokenType::OpPlus => {
                        let val_a: i64 = stack.pop().expect("Error: Failed to pop stack");
                        let val_b: i64 = stack.pop().expect("Error: Failed to pop stack");
                        stack.push(val_a + val_b);
                    },
                    TokenType::OpMinus => {
                        let val_a: i64 = stack.pop().expect("Error: Failed to pop stack");
                        let val_b: i64 = stack.pop().expect("Error: Failed to pop stack");
                        stack.push(val_b - val_a);
                    },
                    TokenType::OpMul => {
                        let val_a: i64 = stack.pop().expect("Error: Failed to pop stack");
                        let val_b: i64 = stack.pop().expect("Error: Failed to pop stack");
                        stack.push(val_a * val_b);
                    },
                    TokenType::OpDiv => {
                        let val_a: i64 = stack.pop().expect("Error: Failed to pop stack");
                        let val_b: i64 = stack.pop().expect("Error: Failed to pop stack");
                        stack.push(val_b / val_a);
                    },
                    _ => unimplemented!("Simulating other bin ops"),
                }
            },
            _ => {
                unimplemented!("Parsing all tokens");
            },
        }
    }
    eprintln!("Simulation Complete");
}

pub fn usage(com: &str, path: &str) -> String {
    format!("
\x1b[31mCOMPILATION FAILED\x1b[0m

\x1b[92mUSAGE:\x1b[0m
  {} \x1b[33m<subcommand>\x1b[0m {}

\x1b[92mSUBCOMMANDS:\x1b[0m
  `\x1b[33mcom\x1b[0m`:   Compile  the program
  `\x1b[33msim\x1b[0m`:   Simulate the program
", com, path)
}                  
                   
pub fn main() {
    let args: Vec<String> = env::args().collect();

    let mut it = args.iter();
    let com: &String = it.next().unwrap_or_else(|| panic!("Error: Failed to read compiler name from command line arguments"));
    let subcom: &String = it.next().unwrap_or_else(|| panic!("{}", usage(com, "\x1b[33m<file path>\x1b[0m")));
    let src_path: &String = it.next().unwrap_or_else(|| panic!("{}", usage(com, "\x1b[33m<file path>\x1b[0m")));

    if it.len() > 0 {
        panic!("{}", usage(com, src_path))
    }

    let src_code: String = fs::read_to_string(src_path).unwrap_or_else(|_| panic!("{}", usage(com, src_path)));
    match subcom.as_str() {
        "com" => compile(src_path.to_string(), src_code),
        "sim" => simulate(src_path.to_string(), src_code),
        _ => panic!("{}", usage(com, src_path)),
    }
}

use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use crate::definitions::TokenType;
use crate::lexer::Lexer;
use crate::parser::NodeType;
use crate::parser::ParseNode;
use crate::parser::ParseTree;

pub mod definitions;
pub mod lexer;
pub mod parser;

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_compilation() {
        let tests: [(&'static str, &'static str); 1] = [
            ("./tests/test_arithmetic.lang", "./tests/test_arithmetic.expected"),
        ];
        for test in tests {
            let src_path = test.0;
            let exp_path = test.1;
            let src: String = fs::read_to_string(src_path).expect("Error: Test failed to read source file");
            let exp: String = fs::read_to_string(exp_path).expect("Error: Test failed to read expected file");

            compile(src_path.to_string(), src.clone(), vec![]);
            let run = Command::new("./output").output().expect("Error: Failed to run executable");
            let stdout = String::from_utf8(run.stdout).expect("Error: Failed to convert stdout to string");
            assert_eq!(exp, stdout, "{} Error: Unexpected Program output.\nExpected:\n{}\n\nGot:\n{}", src_path, exp, stdout);
        }
    }
}

#[derive(PartialEq)]
enum Flag {
    EmitTokens,
    EmitParseTree,
    EmitAsm,
    Run
}

fn generate_nasm_x86(out_path: String, ast: &mut ParseTree) -> std::io::Result<()> {
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

    for statement in &ast.root.children {
        eprintln!("Statement: {} ({:?}", statement.tok.val_str(), statement.kind);
        match statement.kind {
            NodeType::FuncDecl => {
                writeln!(f, "; --- FuncDecl {} ---", statement.tok.val_str())?;
                writeln!(f, "{}:", statement.tok.val_str())?;
            }
            _ => panic!("{} Error: Invalid top level statement `{}`", statement.tok.pos, statement.tok.val_str())
        }

        let mut nodes: Vec<ParseNode> = Vec::new();
        ast.post_order(statement.clone(), &mut nodes);
        nodes.pop();
        for node in nodes {
            match node.kind {
                NodeType::FuncCall => {
                    writeln!(f, "; --- FuncCall {} ---", node.tok.val_str())?;
                    writeln!(f, "    call {}", node.tok.val_str())?;
                },
                NodeType::Literal => {
                    writeln!(f, "; --- Literal {} ---", node.tok.val_str())?;
                    writeln!(f, "    mov rax, {}", node.tok.val_str())?;
                    writeln!(f, "    push rax")?;
                },
                NodeType::Exit => {
                    writeln!(f, "; --- Exit ---")?;
                    writeln!(f, "    pop rdi")?;
                    writeln!(f, "    mov rax, 60")?;
                    writeln!(f, "    syscall")?;
                },
                NodeType::DebugDump => {
                    writeln!(f, "; --- DebugDump ---")?;
                    writeln!(f, "    pop rdi")?;
                    writeln!(f, "    call dump")?;
                },
                NodeType::UnOp => {
                    match node.tok.kind {
                        TokenType::OpMinus => {
                            writeln!(f, "; --- UnOp::OpMinus ---")?;
                            writeln!(f, "    pop rax")?;
                            writeln!(f, "    neg rax")?;
                            writeln!(f, "    push rax")?;
                        },
                        _ => panic!("Error: Unknown unary operator kind `{:?}`", node.tok.kind)
                    }
                },
                NodeType::BinOp => {
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
                    eprintln!("Type: {:?}", node.kind);
                    panic!("{} Error: Invalid node in statement `{}`", node.tok.pos, node.tok.val_str())
                }
            }
        }

        match statement.kind {
            NodeType::FuncDecl => {
                writeln!(f, "; --- Implicit Return ---")?;
                writeln!(f, "    ret")?;
            }
            _ => panic!("{} Error: Invalid top level statement `{}`", statement.tok.pos, statement.tok.val_str())
        }
    }

    writeln!(f, "; --- Footer ---")?;
    writeln!(f, "_start:")?;
    writeln!(f, "    call main")?;
    writeln!(f, "    mov rdi, 0")?;
    writeln!(f, "    mov rax, 60")?;
    writeln!(f, "    syscall")?;

    Ok(())
}

fn compile(src_path: String, src_code: String, flags: Vec<Flag>) {
    eprintln!("\nInfo: Compiling program");

    let mut lexer: Lexer = Lexer::new(src_code);
    lexer.tokenize();
    lexer.lex();
    if flags.contains(&Flag::EmitTokens) {
        eprintln!("Info: Emitting Tokens:");
        for tok in &lexer.toks {
            eprintln!("    Token: {}: {:?} `{}`", tok.pos, tok.kind, tok.val_str());
        }
        eprintln!();
    }
    let ast = &mut parser::ParseTree::new(src_path.clone());
    ast.construct(&mut lexer);
    if flags.contains(&Flag::EmitParseTree) {
        eprintln!("Info: Emitting Parse Tree:");
        ast.dump();
        eprintln!();
    }

    let generate = generate_nasm_x86("output.asm".to_string(), ast);
    let _ = generate.inspect_err(|e| panic!("Error: Failed to generate assembly: {e}"));

    eprintln!("Info: Calling `nasm -f elf64 -o output.o output.asm`");
    let assemble = Command::new("nasm").arg("-f").arg("elf64").arg("-o").arg("output.o").arg("output.asm").output();
    let assemble_err: String = String::from_utf8(assemble.ok().unwrap().stderr).expect("");
    if !assemble_err.is_empty() {
        panic!("\n\x1b[31mCOMPILATION FAILED (assembler) \n{}\x1b[0m", assemble_err);
    }

    eprintln!("Info: Calling `ld -o output output.o`");
    let link = Command::new("ld").arg("-o").arg("output").arg("output.o").output();
    let link_err: String = String::from_utf8(link.ok().unwrap().stderr).expect("");
    if !link_err.is_empty() {
        panic!("\n\x1b[31mCOMPILATION FAILED (linker) \n{}\x1b[0m", link_err);
    }

    if !flags.contains(&Flag::EmitAsm) {
        eprintln!("Info: Calling `rm output.asm`");
        let rm_asm = Command::new("rm").arg("output.asm").output();
        let rm_asm_err: String = String::from_utf8(rm_asm.expect("Error: Failed to retrieve output of assembling").stderr).expect("Error: Failed to convert stderr to string");
        if !rm_asm_err.is_empty() {
            panic!("\n\x1b[31mCOMPILATION FAILED (delete intermediate .asm) \n{}\x1b[0m", rm_asm_err);
        }
    }

    eprintln!("Info: Calling `rm output.o`");
    let rm_o = Command::new("rm").arg("output.o").output();
    let rm_o_err: String = String::from_utf8(rm_o.expect("Error: Failed to retrieve result of linking").stderr).expect("Error: Failed to convert stderr to string");
    if !rm_o_err.is_empty() {
        panic!("\n\x1b[31mCOMPILATION FAILED (delete intermediate .o) \n{}\x1b[0m", rm_o_err);
    }

    eprintln!("\n\x1b[92mCOMPILATION COMPLETE\x1b[0m");

    if flags.contains(&Flag::Run) {
        eprintln!("Info: Calling `./output`");
        let run = Command::new("./output").spawn().expect("Error: Failed to run executable").wait_with_output();
        let status = run.expect("Error: Failed to retrieve output of running").status;
        eprintln!("Info: Exit code {}", status.code().expect("Error: Failed to retrieve exit code of executable"));
    }
}

pub fn usage(com: &str, path: &str) -> String {
    format!("
\x1b[31mCOMPILATION FAILED\x1b[0m

\x1b[92mUSAGE:\x1b[0m
  {} {} \x1b[33m<flags>\x1b[0m 

\x1b[92mFLAGS:\x1b[0m
  \x1b[33m-r     --run\x1b[0m:          Run after compiling
  \x1b[33m-pt    --parse-tree\x1b[0m:   Print parse tree
  \x1b[33m-t     --tokens\x1b[0m:       Print tokens
  \x1b[33m-a     --assembly\x1b[0m:     Keep intermediate assembly
", com, path)
}                  
                   
pub fn main() {
    let args: Vec<String> = env::args().collect();

    let mut it = args.iter();
    let com: &String = it.next().unwrap_or_else(|| panic!("Error: Failed to read compiler name from command line arguments"));
    let src_path: &String = it.next().unwrap_or_else(|| panic!("{}", usage(com, "\x1b[33m<file path>\x1b[0m")));

    let mut flags: Vec<Flag> = Vec::new();
    for arg in it {
        match arg.as_str() {
            "-r" | "--run"         => flags.push(Flag::Run),
            "-a" | "--assembly"    => flags.push(Flag::EmitAsm),
            "-pt" | "--parse-tree" => flags.push(Flag::EmitParseTree),
            "-t" | "--tokens"      => flags.push(Flag::EmitTokens),
            _ => panic!("{}", usage(com, src_path))
        }
    }

    let src_code: String = fs::read_to_string(src_path).unwrap_or_else(|_| panic!("{}", usage(com, src_path)));
    compile(src_path.to_string(), src_code, flags);
}

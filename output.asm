; --- Header ---
global _start
section .text
; --- Debug Dump ---
dump:
    sub rsp, 40
    lea rsi, [rsp + 31]
    mov byte [rsp + 31], 10
    mov ecx, 1
    mov r8, -3689348814741910323
.LBB0_1:
    mov rax, rdi
    mul r8
    shr rdx, 3
    lea eax, [rdx + rdx]
    lea eax, [rax + 4*rax]
    mov r9d, edi
    sub r9d, eax
    or r9b, 48
    mov byte [rsi - 1], r9b
    dec rsi
    inc rcx
    cmp rdi, 9
    mov rdi, rdx
    ja .LBB0_1
    mov edi, 1
    mov rdx, rcx
    mov rax, 1
    syscall
    add rsp, 40
    ret
_start:
; --- Literal 10 ---
    mov rax, 10
    push rax
; --- Literal 10 ---
    mov rax, 10
    push rax
; --- BinOp::OpPlus ---
    pop rax
    pop rbx
    add rax, rbx
    push rax
; --- Literal 5 ---
    mov rax, 5
    push rax
; --- BinOp::OpMinus---
    pop rax
    pop rbx
    sub rbx, rax
    push rbx
; --- DebugDump ---
    pop rdi
    call dump
; --- Literal 2 ---
    mov rax, 2
    push rax
; --- Literal 3 ---
    mov rax, 3
    push rax
; --- BinOp::OpMul ---
    pop rax
    pop rbx
    imul rax, rbx
    push rax
; --- Literal 4 ---
    mov rax, 4
    push rax
; --- Literal 5 ---
    mov rax, 5
    push rax
; --- Literal 2 ---
    mov rax, 2
    push rax
; --- BinOp::OpMul ---
    pop rax
    pop rbx
    imul rax, rbx
    push rax
; --- BinOp::OpPlus ---
    pop rax
    pop rbx
    add rax, rbx
    push rax
; --- BinOp::OpPlus ---
    pop rax
    pop rbx
    add rax, rbx
    push rax
; --- DebugDump ---
    pop rdi
    call dump
; --- Literal 100 ---
    mov rax, 100
    push rax
; --- Literal 10 ---
    mov rax, 10
    push rax
; --- BinOp::OpDiv ---
    pop rcx
    pop rax
    xor rdx, rdx
    idiv rcx
    push rax
; --- Literal 3 ---
    mov rax, 3
    push rax
; --- BinOp::OpPlus ---
    pop rax
    pop rbx
    add rax, rbx
    push rax
; --- DebugDump ---
    pop rdi
    call dump
; --- Literal 69 ---
    mov rax, 69
    push rax
; --- Return ---
    pop rdi
    mov rax, 60
    syscall
; --- Footer ---
    mov rdi, 0
    mov rax, 60
    syscall

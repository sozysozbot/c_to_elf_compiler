GLOBAL _start
SECTION .text
putchar:
        push rbp
        mov rbp, rsp
        sub rsp, 8

        mov [rbp-4], rdi
        
        mov rax, 1 ; write
        mov rdi, 1 ; stdout
        lea rsi, [rbp-4] ; buffer
        mov rdx, 1 ; length
        syscall

        leave
        ret

_start:
        mov rdi, 0x61 ; 'a'
        call    putchar

        mov     edi, 0
        mov     eax, 60
        syscall

GLOBAL _start
SECTION .text
putchar:
        push rbp
        mov rbp, rsp
        sub rsp, 0

        push rdi
        mov rax, 1 ; write
        mov rdi, 1 ; stdout
        mov rsi, rsp ; buffer
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

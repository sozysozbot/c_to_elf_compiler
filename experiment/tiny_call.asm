GLOBAL _start
SECTION .text
_test:
        mov eax, 3
        ret
_start:
        call    _test
        mov     edi, eax

        mov     eax, 60
        syscall

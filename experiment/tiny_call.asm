GLOBAL _start
SECTION .text
_test:
        mov eax, 3
        ret
_start:
        call    _test
        mov     ebx, eax

        mov     eax, 1
        int     0x80

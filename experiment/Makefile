CFLAGS=-std=c17 -Wall -s -Os

3: 3.c
42: 42.c

clean: 
	rm 3
	rm 42

tiny42: tiny42.asm
	nasm -f elf tiny42.asm
	gcc -Wall -s -nostdlib tiny42.o -o tiny42

.PHONY: clean
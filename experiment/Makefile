tiny42: tiny42.asm
	nasm -felf64  --reproducible tiny42.asm
	ld tiny42.o -o tiny42
	strip tiny42

tiny3: tiny3.asm
	nasm -felf64  --reproducible tiny3.asm
	ld tiny3.o -o tiny3
	strip tiny3

clean: 
	rm tiny42 tiny42.o tiny3 tiny3.o

.PHONY: clean
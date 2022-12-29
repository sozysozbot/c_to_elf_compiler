%.o: %.asm
	nasm -felf64  --reproducible $<

%: %.o
	ld $< -o $@
	strip $@

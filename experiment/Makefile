%.o: %.asm
	nasm -felf64 --reproducible $<

%: %.o
	$${BINTOOLS_PREFIX}ld $< -o $@
	$${BINTOOLS_PREFIX}strip $@

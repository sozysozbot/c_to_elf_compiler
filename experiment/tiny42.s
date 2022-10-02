.globl _start
_start:
	movl	$1, %eax
	movl	$42, %ebx  
	int		$0x80

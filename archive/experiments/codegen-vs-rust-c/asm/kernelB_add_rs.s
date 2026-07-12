	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_accumulate
	.p2align	2
_accumulate:
	.cfi_startproc
	cbz	x2, LBB0_2
	ldr	x8, [x0]
	ldr	x9, [x1]
	madd	x8, x9, x2, x8
	str	x8, [x0]
LBB0_2:
	ret
	.cfi_endproc

.subsections_via_symbols

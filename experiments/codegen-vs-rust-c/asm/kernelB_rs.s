	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_accumulate
	.p2align	2
_accumulate:
	.cfi_startproc
	cbz	x2, LBB0_4
	ldr	x8, [x0]
	ldr	x9, [x1]
	mov	x10, #31765
	movk	x10, #32586, lsl #16
	movk	x10, #31161, lsl #32
	movk	x10, #40503, lsl #48
LBB0_2:
	eor	x8, x9, x8
	mul	x8, x8, x10
	subs	x2, x2, #1
	b.ne	LBB0_2
	str	x8, [x0]
LBB0_4:
	ret
	.cfi_endproc

.subsections_via_symbols

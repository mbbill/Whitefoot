	.build_version macos, 26, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_twice_read                     ; -- Begin function twice_read
	.p2align	2
_twice_read:                            ; @twice_read
; %bb.0:                                ; %entry
	ldr	w8, [x0]
	adds	w8, w8, #1
	b.vs	LBB0_2
; %bb.1:                                ; %L5
	str	w8, [x1]
	ldr	w8, [x0]
	lsl	w0, w8, #1
	ret
LBB0_2:                                 ; %trap
	brk	#0x1
                                        ; -- End function
.subsections_via_symbols

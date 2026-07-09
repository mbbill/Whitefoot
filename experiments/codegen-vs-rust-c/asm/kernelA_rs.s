	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.private_extern	__ZN3std2rt10lang_start17hbc047995c31d0a61E
	.globl	__ZN3std2rt10lang_start17hbc047995c31d0a61E
	.p2align	2
__ZN3std2rt10lang_start17hbc047995c31d0a61E:
	.cfi_startproc
	sub	sp, sp, #32
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	mov	x4, x3
	mov	x3, x2
	mov	x2, x1
	str	x0, [sp, #8]
Lloh0:
	adrp	x1, l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.0@PAGE
Lloh1:
	add	x1, x1, l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.0@PAGEOFF
	add	x0, sp, #8
	bl	__ZN3std2rt19lang_start_internal17hd700ba983d3377dcE
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.loh AdrpAdd	Lloh0, Lloh1
	.cfi_endproc

	.p2align	2
__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h283c45f706594cf1E:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x0, [x0]
	bl	__ZN3std3sys9backtrace28__rust_begin_short_backtrace17h281b40cf105699cfE
	mov	w0, #0
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__ZN3std3sys9backtrace28__rust_begin_short_backtrace17h281b40cf105699cfE:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	blr	x0
	; InlineAsm Start
	; InlineAsm End
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h05f5e1a23a51c752E:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x0, [x0]
	bl	__ZN3std3sys9backtrace28__rust_begin_short_backtrace17h281b40cf105699cfE
	mov	w0, #0
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.private_extern	__ZN7kernelA4main17h5e0edf54b52f4d90E
	.globl	__ZN7kernelA4main17h5e0edf54b52f4d90E
	.p2align	2
__ZN7kernelA4main17h5e0edf54b52f4d90E:
	.cfi_startproc
	sub	sp, sp, #96
	.cfi_def_cfa_offset 96
	stp	x29, x30, [sp, #80]
	add	x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	mov	x0, #52719
	movk	x0, #35243, lsl #16
	movk	x0, #17767, lsl #32
	movk	x0, #291, lsl #48
	mov	w1, #49664
	movk	w1, #3051, lsl #16
	bl	_mix_n
	str	x0, [sp, #8]
	add	x8, sp, #8
Lloh2:
	adrp	x9, __ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h14d52cab6e85bc6fE@GOTPAGE
Lloh3:
	ldr	x9, [x9, __ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h14d52cab6e85bc6fE@GOTPAGEOFF]
	stp	x8, x9, [x29, #-16]
Lloh4:
	adrp	x8, l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.2@PAGE
Lloh5:
	add	x8, x8, l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.2@PAGEOFF
	mov	w9, #2
	stp	x8, x9, [sp, #16]
	sub	x8, x29, #16
	mov	w9, #1
	str	x8, [sp, #32]
	stp	x9, xzr, [sp, #40]
	add	x0, sp, #16
	bl	__ZN3std2io5stdio6_print17h31727a912c7756f3E
	ldrb	w0, [sp, #8]
	bl	__ZN3std7process4exit17h39abe55532448edfE
	.loh AdrpAdd	Lloh4, Lloh5
	.loh AdrpLdrGot	Lloh2, Lloh3
	.cfi_endproc

	.globl	_mix_n
	.p2align	2
_mix_n:
	.cfi_startproc
	cbz	x1, LBB5_4
	mov	x8, x0
	mov	x0, #0
	mov	x9, #31765
	movk	x9, #32586, lsl #16
	movk	x9, #31161, lsl #32
	movk	x9, #40503, lsl #48
	mov	x10, #58809
	movk	x10, #7396, lsl #16
	movk	x10, #18285, lsl #32
	movk	x10, #48984, lsl #48
	mov	x11, #4587
	movk	x11, #4913, lsl #16
	movk	x11, #18875, lsl #32
	movk	x11, #38096, lsl #48
LBB5_2:
	add	x8, x8, x9
	eor	x8, x8, x8, lsr #30
	mul	x8, x8, x10
	eor	x8, x8, x8, lsr #27
	mul	x8, x8, x11
	eor	x8, x8, x8, lsr #31
	eor	x0, x8, x0
	subs	x1, x1, #1
	b.ne	LBB5_2
	ret
LBB5_4:
	mov	x0, #0
	ret
	.cfi_endproc

	.globl	_main
	.p2align	2
_main:
	.cfi_startproc
	sub	sp, sp, #32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	mov	x3, x1
	sxtw	x2, w0
Lloh6:
	adrp	x8, __ZN7kernelA4main17h5e0edf54b52f4d90E@PAGE
Lloh7:
	add	x8, x8, __ZN7kernelA4main17h5e0edf54b52f4d90E@PAGEOFF
	str	x8, [sp, #8]
Lloh8:
	adrp	x1, l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.0@PAGE
Lloh9:
	add	x1, x1, l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.0@PAGEOFF
	add	x0, sp, #8
	mov	w4, #0
	bl	__ZN3std2rt19lang_start_internal17hd700ba983d3377dcE
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	ret
	.loh AdrpAdd	Lloh8, Lloh9
	.loh AdrpAdd	Lloh6, Lloh7
	.cfi_endproc

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.0:
	.asciz	"\000\000\000\000\000\000\000\000\b\000\000\000\000\000\000\000\b\000\000\000\000\000\000"
	.quad	__ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h05f5e1a23a51c752E
	.quad	__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h283c45f706594cf1E
	.quad	__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h283c45f706594cf1E

	.section	__TEXT,__const
l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.1:
	.byte	10

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.2:
	.quad	1
	.space	8
	.quad	l_anon.7d98a8f3ea52a7d3b2059ead945cb0f9.1
	.asciz	"\001\000\000\000\000\000\000"

.subsections_via_symbols

	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__ZN12rust_kernels13kernel_rebind17h842e7cdabfaa07e6E:
	.cfi_startproc
	ldr	x8, [x0, #16]
	ldr	x9, [x0, #40]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #64]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #88]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #112]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #136]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #160]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #184]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	cbz	x8, LBB0_6
	stp	x24, x23, [sp, #-48]!
	.cfi_def_cfa_offset 48
	stp	x22, x21, [sp, #16]
	stp	x20, x19, [sp, #32]
	.cfi_offset w19, -8
	.cfi_offset w20, -16
	.cfi_offset w21, -24
	.cfi_offset w22, -32
	.cfi_offset w23, -40
	.cfi_offset w24, -48
	.cfi_remember_state
	ldr	x9, [x0, #8]
	ldr	x13, [x0, #32]
	ldr	x10, [x0, #56]
	ldr	x11, [x0, #80]
	ldr	x12, [x0, #104]
	ldr	x14, [x0, #128]
	ldr	x15, [x0, #152]
	ldr	x16, [x0, #176]
	cmp	x8, #16
	b.hs	LBB0_7
	mov	x17, #0
LBB0_3:
	mov	x0, #0
	lsl	x1, x17, #3
	add	x16, x16, x1
	add	x15, x15, x1
	add	x13, x13, x1
	add	x14, x14, x1
	add	x12, x12, x1
	add	x11, x11, x1
	add	x10, x10, x1
	add	x9, x9, x1
	sub	x8, x8, x17
LBB0_4:
	ldr	x17, [x9, x0, lsl #3]
	ldr	x1, [x10, x0, lsl #3]
	add	x17, x1, x17
	ldr	x1, [x11, x0, lsl #3]
	ldr	x2, [x12, x0, lsl #3]
	add	x1, x1, x2
	add	x17, x17, x1
	ldr	x1, [x14, x0, lsl #3]
	add	x17, x17, x1
	str	x17, [x9, x0, lsl #3]
	ldr	x17, [x13, x0, lsl #3]
	ldr	x1, [x12, x0, lsl #3]
	ldr	x2, [x14, x0, lsl #3]
	add	x17, x1, x17
	ldr	x1, [x15, x0, lsl #3]
	ldr	x3, [x16, x0, lsl #3]
	add	x1, x2, x1
	add	x17, x17, x1
	add	x17, x17, x3
	str	x17, [x13, x0, lsl #3]
	add	x0, x0, #1
	cmp	x8, x0
	b.ne	LBB0_4
LBB0_5:
	ldp	x20, x19, [sp, #32]
	ldp	x22, x21, [sp, #16]
	ldp	x24, x23, [sp], #48
	.cfi_def_cfa_offset 0
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
LBB0_6:
	ret
LBB0_7:
	.cfi_restore_state
	mov	x17, #0
	lsl	x5, x8, #3
	add	x23, x9, x5
	add	x24, x13, x5
	add	x6, x10, x5
	cmp	x9, x6
	ccmp	x10, x23, #2, lo
	cset	w0, lo
	add	x7, x11, x5
	cmp	x9, x7
	ccmp	x11, x23, #2, lo
	cset	w1, lo
	add	x19, x12, x5
	cmp	x9, x19
	ccmp	x12, x23, #2, lo
	cset	w2, lo
	add	x20, x14, x5
	cmp	x9, x20
	ccmp	x14, x23, #2, lo
	cset	w3, lo
	add	x21, x15, x5
	cmp	x9, x21
	ccmp	x15, x23, #2, lo
	cset	w4, lo
	add	x22, x16, x5
	cmp	x9, x22
	ccmp	x16, x23, #2, lo
	cset	w5, lo
	cmp	x13, x6
	ccmp	x10, x24, #2, lo
	cset	w6, lo
	cmp	x13, x7
	ccmp	x11, x24, #2, lo
	cset	w7, lo
	cmp	x13, x19
	ccmp	x12, x24, #2, lo
	cset	w19, lo
	cmp	x13, x20
	ccmp	x14, x24, #2, lo
	cset	w20, lo
	cmp	x13, x21
	ccmp	x15, x24, #2, lo
	cset	w21, lo
	cmp	x13, x22
	ccmp	x16, x24, #2, lo
	cset	w22, lo
	cmp	x13, x23
	ccmp	x9, x24, #2, lo
	b.lo	LBB0_3
	tbnz	w0, #0, LBB0_3
	tbnz	w1, #0, LBB0_3
	tbnz	w2, #0, LBB0_3
	tbnz	w3, #0, LBB0_3
	tbnz	w4, #0, LBB0_3
	tbnz	w5, #0, LBB0_3
	tbnz	w6, #0, LBB0_3
	tbnz	w7, #0, LBB0_3
	tbnz	w19, #0, LBB0_3
	tbnz	w20, #0, LBB0_3
	tbnz	w21, #0, LBB0_3
	tbnz	w22, #0, LBB0_3
	and	x17, x8, #0xffffffffffffffe
	mov	x0, x9
	mov	x1, x10
	mov	x2, x11
	mov	x3, x12
	mov	x4, x14
	mov	x5, x13
	mov	x6, x15
	mov	x7, x16
	mov	x19, x17
LBB0_21:
	ldr	q0, [x0]
	ldr	q1, [x1], #16
	add.2d	v0, v1, v0
	ldr	q1, [x2], #16
	ldr	q2, [x3], #16
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	ldr	q1, [x4], #16
	add.2d	v0, v0, v1
	str	q0, [x0], #16
	ldr	q0, [x5]
	add.2d	v0, v2, v0
	ldr	q2, [x6], #16
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	ldr	q1, [x7], #16
	add.2d	v0, v0, v1
	str	q0, [x5], #16
	subs	x19, x19, #2
	b.ne	LBB0_21
	cmp	x8, x17
	b.ne	LBB0_3
	b	LBB0_5
	.cfi_endproc

	.p2align	2
__ZN12rust_kernels14kernel_innerfn17h8bfca1a0dbfe6d64E:
	.cfi_startproc
	sub	sp, sp, #80
	.cfi_def_cfa_offset 80
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldp	x8, x1, [x0, #8]
	ldp	x2, x3, [x0, #32]
	ldp	x4, x5, [x0, #56]
	ldp	x6, x7, [x0, #80]
	ldur	q0, [x0, #104]
	ldr	q1, [x0, #128]
	ldur	q2, [x0, #152]
	ldp	x9, x10, [x0, #176]
	stp	x9, x10, [sp, #48]
	stp	q1, q2, [sp, #16]
	str	q0, [sp]
	mov	x0, x8
	bl	__ZN12rust_kernels5inner17he7900cc6e13749c7E
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	add	sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__ZN12rust_kernels14kernel_obvious17ha02a80708fbb71a5E:
	.cfi_startproc
	ldr	x8, [x0, #16]
	ldr	x9, [x0, #40]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #64]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #88]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #112]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #136]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #160]
	cmp	x9, x8
	csel	x8, x9, x8, lo
	ldr	x9, [x0, #184]
	cmp	x9, x8
	csel	x17, x9, x8, lo
	cbz	x17, LBB2_6
	stp	x26, x25, [sp, #-64]!
	.cfi_def_cfa_offset 64
	stp	x24, x23, [sp, #16]
	stp	x22, x21, [sp, #32]
	stp	x20, x19, [sp, #48]
	.cfi_offset w19, -8
	.cfi_offset w20, -16
	.cfi_offset w21, -24
	.cfi_offset w22, -32
	.cfi_offset w23, -40
	.cfi_offset w24, -48
	.cfi_offset w25, -56
	.cfi_offset w26, -64
	.cfi_remember_state
	ldr	x8, [x0, #8]
	ldr	x9, [x0, #56]
	ldr	x10, [x0, #80]
	ldr	x11, [x0, #104]
	ldr	x12, [x0, #128]
	ldr	x13, [x0, #32]
	ldr	x14, [x0, #152]
	ldr	x15, [x0, #176]
	cmp	x17, #16
	b.hs	LBB2_7
	mov	x16, #0
	mov	w0, #1
LBB2_3:
	add	x17, x17, #1
LBB2_4:
	ldr	x1, [x8, x16, lsl #3]
	ldr	x2, [x9, x16, lsl #3]
	ldr	x3, [x10, x16, lsl #3]
	add	x1, x2, x1
	ldr	x2, [x11, x16, lsl #3]
	ldr	x4, [x12, x16, lsl #3]
	add	x2, x3, x2
	add	x1, x1, x2
	add	x1, x1, x4
	str	x1, [x8, x16, lsl #3]
	ldr	x1, [x13, x16, lsl #3]
	ldr	x2, [x11, x16, lsl #3]
	add	x1, x2, x1
	ldr	x2, [x12, x16, lsl #3]
	ldr	x3, [x14, x16, lsl #3]
	add	x2, x2, x3
	add	x1, x1, x2
	ldr	x2, [x15, x16, lsl #3]
	add	x1, x1, x2
	str	x1, [x13, x16, lsl #3]
	mov	x16, x0
	add	x0, x0, #1
	cmp	x17, x0
	b.ne	LBB2_4
LBB2_5:
	ldp	x20, x19, [sp, #48]
	ldp	x22, x21, [sp, #32]
	ldp	x24, x23, [sp, #16]
	ldp	x26, x25, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
LBB2_6:
	ret
LBB2_7:
	.cfi_restore_state
	mov	x16, #0
	lsl	x0, x17, #3
	add	x24, x8, x0
	add	x25, x13, x0
	add	x7, x9, x0
	cmp	x8, x7
	ccmp	x9, x24, #2, lo
	cset	w1, lo
	add	x19, x10, x0
	cmp	x8, x19
	ccmp	x10, x24, #2, lo
	cset	w2, lo
	add	x20, x11, x0
	cmp	x8, x20
	ccmp	x11, x24, #2, lo
	cset	w3, lo
	add	x21, x12, x0
	cmp	x8, x21
	ccmp	x12, x24, #2, lo
	cset	w4, lo
	add	x22, x14, x0
	cmp	x8, x22
	ccmp	x14, x24, #2, lo
	cset	w5, lo
	add	x0, x15, x0
	cmp	x8, x0
	ccmp	x15, x24, #2, lo
	cset	w6, lo
	cmp	x13, x7
	ccmp	x9, x25, #2, lo
	cset	w7, lo
	cmp	x13, x19
	ccmp	x10, x25, #2, lo
	cset	w19, lo
	cmp	x13, x20
	ccmp	x11, x25, #2, lo
	cset	w20, lo
	cmp	x13, x21
	ccmp	x12, x25, #2, lo
	cset	w21, lo
	cmp	x13, x22
	ccmp	x14, x25, #2, lo
	cset	w23, lo
	cmp	x13, x0
	ccmp	x15, x25, #2, lo
	cset	w22, lo
	cmp	x13, x24
	ccmp	x8, x25, #2, lo
	mov	w0, #1
	b.lo	LBB2_3
	tbnz	w1, #0, LBB2_3
	tbnz	w2, #0, LBB2_3
	tbnz	w3, #0, LBB2_3
	tbnz	w4, #0, LBB2_3
	tbnz	w5, #0, LBB2_3
	tbnz	w6, #0, LBB2_3
	tbnz	w7, #0, LBB2_3
	tbnz	w19, #0, LBB2_3
	tbnz	w20, #0, LBB2_3
	tbnz	w21, #0, LBB2_3
	tbnz	w23, #0, LBB2_3
	tbnz	w22, #0, LBB2_3
	and	x16, x17, #0xffffffffffffffe
	orr	x0, x17, #0x1
	mov	x1, x8
	mov	x2, x9
	mov	x3, x10
	mov	x4, x11
	mov	x5, x12
	mov	x6, x13
	mov	x7, x14
	mov	x19, x15
	mov	x20, x16
LBB2_21:
	ldr	q0, [x1]
	ldr	q1, [x2], #16
	add.2d	v0, v1, v0
	ldr	q1, [x3], #16
	ldr	q2, [x4], #16
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	ldr	q1, [x5], #16
	add.2d	v0, v0, v1
	str	q0, [x1], #16
	ldr	q0, [x6]
	add.2d	v0, v2, v0
	ldr	q2, [x7], #16
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	ldr	q1, [x19], #16
	add.2d	v0, v0, v1
	str	q0, [x6], #16
	subs	x20, x20, #2
	b.ne	LBB2_21
	cmp	x17, x16
	b.ne	LBB2_3
	b	LBB2_5
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI3_0:
	.quad	0
	.quad	1
	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__ZN12rust_kernels3run17h0f560483664fa09bE:
Lfunc_begin0:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception0
	stp	x28, x27, [sp, #-96]!
	.cfi_def_cfa_offset 96
	stp	x26, x25, [sp, #16]
	stp	x24, x23, [sp, #32]
	stp	x22, x21, [sp, #48]
	stp	x20, x19, [sp, #64]
	stp	x29, x30, [sp, #80]
	add	x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_remember_state
	sub	sp, sp, #464
	mov	x21, #0
	stp	x0, x1, [sp, #72]
	stp	x2, x3, [sp, #88]
	lsl	x25, x2, #3
	lsr	x8, x2, #61
	cbnz	x8, LBB3_7
	mov	x8, #9223372036854775800
	cmp	x25, x8
	b.hi	LBB3_7
	mov	x22, x4
	mov	x19, x3
	mov	x20, x2
	cbz	x25, LBB3_8
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	w21, #8
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_7
	mov	x28, x0
	mov	x9, x20
	str	x0, [sp, #64]
	cbz	x20, LBB3_9
LBB3_5:
	str	x9, [sp, #56]
	cmp	x20, #8
	b.hs	LBB3_10
	mov	x8, #0
	b	LBB3_13
LBB3_7:
Lloh0:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh1:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	x0, x21
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
LBB3_8:
	mov	x9, #0
	mov	w28, #8
	str	x28, [sp, #64]
	cbnz	x20, LBB3_5
LBB3_9:
	mov	x8, #0
	mov	x15, #0
	mov	x14, #0
	mov	x13, #0
	mov	x12, #0
	mov	x11, #0
	mov	x10, #0
	mov	w0, #8
	mov	w21, #8
	mov	w28, #8
	mov	w27, #8
	mov	w26, #8
	mov	w24, #8
	mov	w23, #8
	b	LBB3_23
LBB3_10:
	and	x8, x20, #0x1ffffffffffffff8
Lloh2:
	adrp	x9, lCPI3_0@PAGE
Lloh3:
	ldr	q0, [x9, lCPI3_0@PAGEOFF]
	add	x9, x28, #32
	mov	w10, #1
	dup.2d	v1, x10
	mov	w10, #3
	dup.2d	v2, x10
	mov	w10, #5
	dup.2d	v3, x10
	mov	w10, #7
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB3_11:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB3_11
	cmp	x20, x8
	b.eq	LBB3_14
LBB3_13:
	add	x9, x8, #1
	str	x9, [x28, x8, lsl #3]
	mov	x8, x9
	cmp	x20, x9
	b.ne	LBB3_13
LBB3_14:
	cbz	x25, LBB3_18
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_21
	mov	x23, x0
	mov	x21, x20
	cmp	x20, #8
	b.hs	LBB3_19
LBB3_17:
	mov	x8, #0
	b	LBB3_50
LBB3_18:
	mov	x21, #0
	mov	w23, #8
	cmp	x20, #8
	b.lo	LBB3_17
LBB3_19:
	and	x8, x20, #0x1ffffffffffffff8
Lloh4:
	adrp	x9, lCPI3_0@PAGE
Lloh5:
	ldr	q0, [x9, lCPI3_0@PAGEOFF]
	add	x9, x23, #32
	mov	w10, #2
	dup.2d	v1, x10
	mov	w10, #4
	dup.2d	v2, x10
	mov	w10, #6
	dup.2d	v3, x10
	mov	w10, #8
	dup.2d	v4, x10
	mov	x10, x8
LBB3_20:
	add.2d	v5, v0, v1
	add.2d	v6, v0, v2
	add.2d	v7, v0, v3
	stp	q5, q6, [x9, #-32]
	add.2d	v0, v0, v4
	stp	q7, q0, [x9], #64
	subs	x10, x10, #8
	b.ne	LBB3_20
	b	LBB3_51
LBB3_21:
Ltmp0:
Lloh6:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh7:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	w0, #8
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp1:
	b	LBB3_61
LBB3_22:
Ltmp2:
	mov	x19, x0
	b	LBB3_62
LBB3_23:
	str	x9, [sp, #104]
	ldr	x9, [sp, #64]
	stp	x9, x20, [sp, #112]
	stp	x10, x23, [sp, #128]
	stp	x20, x11, [sp, #144]
	stp	x24, x20, [sp, #160]
	stp	x12, x26, [sp, #176]
	stp	x20, x13, [sp, #192]
	stp	x27, x20, [sp, #208]
	stp	x14, x28, [sp, #224]
	stp	x20, x15, [sp, #240]
	stp	x21, x20, [sp, #256]
	stp	x8, x0, [sp, #272]
	mov	w21, #101
	str	x20, [sp, #288]
LBB3_24:
	subs	w21, w21, #1
	b.eq	LBB3_26
Ltmp21:
	add	x0, sp, #104
	blr	x22
Ltmp22:
	b	LBB3_24
LBB3_26:
Ltmp24:
	bl	__ZN3std4time7Instant3now17h96378d48b1d625ebE
Ltmp25:
	stur	x0, [x29, #-248]
	stur	w1, [x29, #-240]
	mov	x21, #-1
	add	x23, sp, #104
	sub	x24, x29, #168
LBB3_28:
	add	x21, x21, #1
	cmp	x21, x19
	b.hs	LBB3_30
	stur	x23, [x29, #-168]
	; InlineAsm Start
	; InlineAsm End
	ldur	x0, [x29, #-168]
Ltmp33:
	blr	x22
Ltmp34:
	b	LBB3_28
LBB3_30:
Ltmp26:
	sub	x0, x29, #248
	bl	__ZN3std4time7Instant7elapsed17h304c2773e294bb50E
Ltmp27:
	cbz	x20, LBB3_36
	ldr	x8, [sp, #120]
	ldr	x9, [sp, #144]
	sub	x10, x20, #1
	cmp	x9, x10
	csel	x10, x9, x10, lo
	cmp	x8, x10
	b.ls	LBB3_44
	cmp	x9, x10
	b.eq	LBB3_45
	ldr	x9, [sp, #136]
	ldr	x8, [sp, #112]
	cmp	x20, #8
	b.hs	LBB3_37
	mov	x21, #0
	mov	x10, #0
	b	LBB3_40
LBB3_36:
	mov	x21, #0
	b	LBB3_42
LBB3_37:
	and	x10, x20, #0x1ffffffffffffff8
	add	x11, x8, #32
	add	x12, x9, #32
	movi.2d	v0, #0000000000000000
	mov	x13, x10
	movi.2d	v1, #0000000000000000
	movi.2d	v2, #0000000000000000
	movi.2d	v3, #0000000000000000
LBB3_38:
	ldp	q4, q5, [x11, #-32]
	ldp	q6, q7, [x11], #64
	ldp	q16, q17, [x12, #-32]
	ldp	q18, q19, [x12], #64
	eor.16b	v4, v16, v4
	eor.16b	v5, v17, v5
	eor.16b	v6, v18, v6
	eor.16b	v7, v19, v7
	add.2d	v0, v4, v0
	add.2d	v1, v5, v1
	add.2d	v2, v6, v2
	add.2d	v3, v7, v3
	subs	x13, x13, #8
	b.ne	LBB3_38
	add.2d	v0, v1, v0
	add.2d	v0, v2, v0
	add.2d	v0, v3, v0
	addp.2d	d0, v0
	fmov	x21, d0
	cmp	x20, x10
	b.eq	LBB3_42
LBB3_40:
	lsl	x11, x10, #3
	add	x9, x9, x11
	add	x8, x8, x11
	sub	x10, x20, x10
LBB3_41:
	ldr	x11, [x8], #8
	ldr	x12, [x9], #8
	eor	x11, x12, x11
	add	x21, x11, x21
	subs	x10, x10, #1
	b.ne	LBB3_41
LBB3_42:
	mov	w8, #51712
	movk	w8, #15258, lsl #16
	umulh	x9, x0, x8
	mul	x8, x0, x8
	adds	x0, x8, w1, uxtw
	cinc	x1, x9, hs
	bl	___floatuntidf
	ucvtf	d1, x20
	stur	x21, [x29, #-232]
	ucvtf	d2, x19
	fmul	d1, d1, d2
	fdiv	d0, d0, d1
	stur	d0, [x29, #-176]
Lloh8:
	adrp	x8, __ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h55043baee9cf6639E@GOTPAGE
Lloh9:
	ldr	x8, [x8, __ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h55043baee9cf6639E@GOTPAGEOFF]
	add	x9, sp, #72
Lloh10:
	adrp	x10, __ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h18cb492d659b6813E@PAGE
Lloh11:
	add	x10, x10, __ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h18cb492d659b6813E@PAGEOFF
	stp	x9, x10, [x29, #-168]
	add	x9, sp, #88
	stp	x9, x8, [x29, #-152]
	add	x9, sp, #96
	stp	x9, x8, [x29, #-136]
	sub	x8, x29, #176
Lloh12:
	adrp	x9, __ZN4core3fmt5float52_$LT$impl$u20$core..fmt..Display$u20$for$u20$f64$GT$3fmt17h233df897a8cc50caE@GOTPAGE
Lloh13:
	ldr	x9, [x9, __ZN4core3fmt5float52_$LT$impl$u20$core..fmt..Display$u20$for$u20$f64$GT$3fmt17h233df897a8cc50caE@GOTPAGEOFF]
	stp	x8, x9, [x29, #-120]
Lloh14:
	adrp	x8, __ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h14d52cab6e85bc6fE@GOTPAGE
Lloh15:
	ldr	x8, [x8, __ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h14d52cab6e85bc6fE@GOTPAGEOFF]
	sub	x9, x29, #232
Lloh16:
	adrp	x10, l_anon.e5f937a298947f43ee04ff650c15395e.6@PAGE
Lloh17:
	add	x10, x10, l_anon.e5f937a298947f43ee04ff650c15395e.6@PAGEOFF
	stp	x9, x8, [x29, #-104]
	mov	w8, #6
	stp	x10, x8, [x29, #-224]
Lloh18:
	adrp	x8, l_anon.e5f937a298947f43ee04ff650c15395e.7@PAGE
Lloh19:
	add	x8, x8, l_anon.e5f937a298947f43ee04ff650c15395e.7@PAGEOFF
	mov	w9, #5
	stp	x8, x9, [x29, #-192]
	sub	x8, x29, #168
	stp	x8, x9, [x29, #-208]
Ltmp30:
	sub	x0, x29, #224
	bl	__ZN3std2io5stdio6_print17h31727a912c7756f3E
Ltmp31:
	add	x0, sp, #104
	bl	__ZN4core3ptr39drop_in_place$LT$rust_kernels..Cols$GT$17h1f70d29370f747a4E
	add	sp, sp, #464
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	x24, x23, [sp, #32]
	ldp	x26, x25, [sp, #16]
	ldp	x28, x27, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	ret
LBB3_44:
	.cfi_restore_state
Lloh20:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.8@PAGE
Lloh21:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.8@PAGEOFF
	b	LBB3_46
LBB3_45:
Lloh22:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.9@PAGE
Lloh23:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.9@PAGEOFF
	mov	x8, x9
LBB3_46:
Ltmp28:
	mov	x0, x8
	mov	x1, x8
	bl	__ZN4core9panicking18panic_bounds_check17h8935b4e0a545757eE
Ltmp29:
	b	LBB3_61
LBB3_47:
Ltmp32:
	mov	x19, x0
	add	x0, sp, #104
	bl	__ZN4core3ptr39drop_in_place$LT$rust_kernels..Cols$GT$17h1f70d29370f747a4E
	mov	x0, x19
	bl	__Unwind_Resume
LBB3_48:
Ltmp35:
	mov	x19, x0
	add	x0, sp, #104
	bl	__ZN4core3ptr39drop_in_place$LT$rust_kernels..Cols$GT$17h1f70d29370f747a4E
	mov	x0, x19
	bl	__Unwind_Resume
LBB3_49:
Ltmp23:
	mov	x19, x0
	add	x0, sp, #104
	bl	__ZN4core3ptr39drop_in_place$LT$rust_kernels..Cols$GT$17h1f70d29370f747a4E
	mov	x0, x19
	bl	__Unwind_Resume
LBB3_50:
	add	x9, x8, #2
	str	x9, [x23, x8, lsl #3]
	add	x8, x8, #1
LBB3_51:
	cmp	x20, x8
	b.ne	LBB3_50
	str	x21, [sp, #48]
	cbz	x25, LBB3_56
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_59
	mov	x24, x0
	mov	x27, x20
	cmp	x20, #8
	b.hs	LBB3_57
LBB3_55:
	mov	x8, #0
	b	LBB3_65
LBB3_56:
	mov	x27, #0
	mov	w24, #8
	cmp	x20, #8
	b.lo	LBB3_55
LBB3_57:
	and	x8, x20, #0x1ffffffffffffff8
Lloh24:
	adrp	x9, lCPI3_0@PAGE
Lloh25:
	ldr	q0, [x9, lCPI3_0@PAGEOFF]
	add	x9, x24, #32
	mov	w10, #3
	dup.2d	v1, x10
	mov	w10, #5
	dup.2d	v2, x10
	mov	w10, #7
	dup.2d	v3, x10
	mov	w10, #9
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB3_58:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB3_58
	b	LBB3_66
LBB3_59:
Ltmp3:
Lloh26:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh27:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	w0, #8
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp4:
	b	LBB3_61
LBB3_60:
Ltmp5:
	mov	x19, x0
	b	LBB3_76
LBB3_61:
	brk	#0x1
LBB3_62:
	ldr	x8, [sp, #56]
	cbz	x8, LBB3_64
	ldp	x8, x0, [sp, #56]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB3_64:
	mov	x0, x19
	bl	__Unwind_Resume
LBB3_65:
	add	x9, x8, #3
	str	x9, [x24, x8, lsl #3]
	add	x8, x8, #1
LBB3_66:
	cmp	x20, x8
	b.ne	LBB3_65
	str	x27, [sp, #40]
	cbz	x25, LBB3_71
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_74
	mov	x26, x0
	str	x20, [sp, #32]
	cmp	x20, #8
	b.hs	LBB3_72
LBB3_70:
	mov	x8, #0
	b	LBB3_78
LBB3_71:
	mov	x8, #0
	mov	w26, #8
	str	x8, [sp, #32]
	cmp	x20, #8
	b.lo	LBB3_70
LBB3_72:
	and	x8, x20, #0x1ffffffffffffff8
Lloh28:
	adrp	x9, lCPI3_0@PAGE
Lloh29:
	ldr	q2, [x9, lCPI3_0@PAGEOFF]
	add	x9, x26, #32
	mov	w10, #4
	dup.2d	v0, x10
	mov	w10, #6
	dup.2d	v1, x10
	mov	w10, #8
	dup.2d	v3, x10
	mov	w10, #10
	dup.2d	v4, x10
	mov	x10, x8
LBB3_73:
	add.2d	v5, v2, v0
	add.2d	v6, v2, v1
	add.2d	v7, v2, v4
	stp	q5, q6, [x9, #-32]
	add.2d	v2, v2, v3
	stp	q2, q7, [x9], #64
	subs	x10, x10, #8
	b.ne	LBB3_73
	b	LBB3_79
LBB3_74:
Ltmp6:
Lloh30:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh31:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	w0, #8
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp7:
	b	LBB3_61
LBB3_75:
Ltmp8:
	mov	x19, x0
	b	LBB3_89
LBB3_76:
	ldr	x8, [sp, #48]
	cbz	x8, LBB3_62
	ldr	x8, [sp, #48]
	lsl	x1, x8, #3
	mov	x0, x23
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB3_62
LBB3_78:
	add	x9, x8, #4
	str	x9, [x26, x8, lsl #3]
	add	x8, x8, #1
LBB3_79:
	cmp	x20, x8
	b.ne	LBB3_78
	cbz	x25, LBB3_84
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_87
	mov	x27, x0
	str	x20, [sp, #24]
	cmp	x20, #8
	b.hs	LBB3_85
LBB3_83:
	mov	x8, #0
	b	LBB3_91
LBB3_84:
	mov	x8, #0
	mov	w27, #8
	str	x8, [sp, #24]
	cmp	x20, #8
	b.lo	LBB3_83
LBB3_85:
	and	x8, x20, #0x1ffffffffffffff8
Lloh32:
	adrp	x9, lCPI3_0@PAGE
Lloh33:
	ldr	q0, [x9, lCPI3_0@PAGEOFF]
	add	x9, x27, #32
	mov	w10, #5
	dup.2d	v1, x10
	mov	w10, #7
	dup.2d	v2, x10
	mov	w10, #9
	dup.2d	v3, x10
	mov	w10, #11
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB3_86:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB3_86
	b	LBB3_92
LBB3_87:
Ltmp9:
Lloh34:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh35:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	w0, #8
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp10:
	b	LBB3_61
LBB3_88:
Ltmp11:
	mov	x19, x0
	b	LBB3_102
LBB3_89:
	ldr	x8, [sp, #40]
	cbz	x8, LBB3_76
	ldr	x8, [sp, #40]
	lsl	x1, x8, #3
	mov	x0, x24
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB3_76
LBB3_91:
	add	x9, x8, #5
	str	x9, [x27, x8, lsl #3]
	add	x8, x8, #1
LBB3_92:
	cmp	x20, x8
	b.ne	LBB3_91
	cbz	x25, LBB3_97
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_100
	mov	x28, x0
	str	x20, [sp, #16]
	cmp	x20, #8
	b.hs	LBB3_98
LBB3_96:
	mov	x8, #0
	b	LBB3_104
LBB3_97:
	mov	x8, #0
	mov	w28, #8
	str	x8, [sp, #16]
	cmp	x20, #8
	b.lo	LBB3_96
LBB3_98:
	and	x8, x20, #0x1ffffffffffffff8
Lloh36:
	adrp	x9, lCPI3_0@PAGE
Lloh37:
	ldr	q3, [x9, lCPI3_0@PAGEOFF]
	add	x9, x28, #32
	mov	w10, #6
	dup.2d	v0, x10
	mov	w10, #8
	dup.2d	v1, x10
	mov	w10, #10
	dup.2d	v2, x10
	mov	w10, #12
	dup.2d	v4, x10
	mov	x10, x8
LBB3_99:
	add.2d	v5, v3, v0
	add.2d	v6, v3, v2
	add.2d	v7, v3, v4
	add.2d	v3, v3, v1
	stp	q5, q3, [x9, #-32]
	stp	q6, q7, [x9], #64
	subs	x10, x10, #8
	b.ne	LBB3_99
	b	LBB3_105
LBB3_100:
Ltmp12:
Lloh38:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh39:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	w0, #8
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp13:
	b	LBB3_61
LBB3_101:
Ltmp14:
	mov	x19, x0
	b	LBB3_115
LBB3_102:
	ldr	x8, [sp, #32]
	cbz	x8, LBB3_89
	ldr	x8, [sp, #32]
	lsl	x1, x8, #3
	mov	x0, x26
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB3_89
LBB3_104:
	add	x9, x8, #6
	str	x9, [x28, x8, lsl #3]
	add	x8, x8, #1
LBB3_105:
	cmp	x20, x8
	b.ne	LBB3_104
	cbz	x25, LBB3_110
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_113
	mov	x21, x0
	mov	x15, x20
	cmp	x20, #8
	b.hs	LBB3_111
LBB3_109:
	mov	x8, #0
	b	LBB3_117
LBB3_110:
	mov	x15, #0
	mov	w21, #8
	cmp	x20, #8
	b.lo	LBB3_109
LBB3_111:
	and	x8, x20, #0x1ffffffffffffff8
Lloh40:
	adrp	x9, lCPI3_0@PAGE
Lloh41:
	ldr	q0, [x9, lCPI3_0@PAGEOFF]
	add	x9, x21, #32
	mov	w10, #7
	dup.2d	v1, x10
	mov	w10, #9
	dup.2d	v2, x10
	mov	w10, #11
	dup.2d	v3, x10
	mov	w10, #13
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB3_112:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB3_112
	b	LBB3_118
LBB3_113:
Ltmp15:
Lloh42:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh43:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	w0, #8
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp16:
	b	LBB3_61
LBB3_114:
Ltmp17:
	mov	x19, x0
	b	LBB3_129
LBB3_115:
	ldr	x8, [sp, #24]
	cbz	x8, LBB3_102
	ldr	x8, [sp, #24]
	lsl	x1, x8, #3
	mov	x0, x27
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB3_102
LBB3_117:
	add	x9, x8, #7
	str	x9, [x21, x8, lsl #3]
	add	x8, x8, #1
LBB3_118:
	cmp	x20, x8
	b.ne	LBB3_117
	cbz	x25, LBB3_123
	str	x15, [sp, #8]
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x25
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB3_126
	mov	x8, x20
	ldr	x15, [sp, #8]
	cmp	x20, #8
	b.hs	LBB3_124
LBB3_122:
	mov	x9, #0
	b	LBB3_131
LBB3_123:
	mov	x8, #0
	mov	w0, #8
	cmp	x20, #8
	b.lo	LBB3_122
LBB3_124:
	and	x9, x20, #0x1ffffffffffffff8
Lloh44:
	adrp	x10, lCPI3_0@PAGE
Lloh45:
	ldr	q3, [x10, lCPI3_0@PAGEOFF]
	add	x10, x0, #32
	mov	w11, #8
	dup.2d	v0, x11
	mov	w11, #10
	dup.2d	v1, x11
	mov	w11, #12
	dup.2d	v2, x11
	mov	w11, #14
	dup.2d	v4, x11
	mov	x11, x9
LBB3_125:
	add.2d	v5, v3, v1
	add.2d	v6, v3, v2
	add.2d	v7, v3, v4
	add.2d	v3, v3, v0
	stp	q3, q5, [x10, #-32]
	stp	q6, q7, [x10], #64
	subs	x11, x11, #8
	b.ne	LBB3_125
	b	LBB3_132
LBB3_126:
Ltmp18:
Lloh46:
	adrp	x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGE
Lloh47:
	add	x2, x2, l_anon.e5f937a298947f43ee04ff650c15395e.15@PAGEOFF
	mov	w0, #8
	mov	x1, x25
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp19:
	b	LBB3_61
LBB3_127:
Ltmp20:
	mov	x19, x0
	ldr	x8, [sp, #8]
	cbz	x8, LBB3_129
	ldr	x8, [sp, #8]
	lsl	x1, x8, #3
	mov	x0, x21
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB3_129:
	ldr	x8, [sp, #16]
	cbz	x8, LBB3_115
	ldr	x8, [sp, #16]
	lsl	x1, x8, #3
	mov	x0, x28
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB3_115
LBB3_131:
	add	x10, x9, #8
	str	x10, [x0, x9, lsl #3]
	add	x9, x9, #1
LBB3_132:
	cmp	x20, x9
	b.ne	LBB3_131
	ldp	x10, x9, [sp, #48]
	ldp	x12, x11, [sp, #32]
	ldp	x14, x13, [sp, #16]
	b	LBB3_23
	.loh AdrpAdd	Lloh0, Lloh1
	.loh AdrpLdr	Lloh2, Lloh3
	.loh AdrpLdr	Lloh4, Lloh5
	.loh AdrpAdd	Lloh6, Lloh7
	.loh AdrpAdd	Lloh18, Lloh19
	.loh AdrpAdd	Lloh16, Lloh17
	.loh AdrpLdrGot	Lloh14, Lloh15
	.loh AdrpLdrGot	Lloh12, Lloh13
	.loh AdrpAdd	Lloh10, Lloh11
	.loh AdrpLdrGot	Lloh8, Lloh9
	.loh AdrpAdd	Lloh20, Lloh21
	.loh AdrpAdd	Lloh22, Lloh23
	.loh AdrpLdr	Lloh24, Lloh25
	.loh AdrpAdd	Lloh26, Lloh27
	.loh AdrpLdr	Lloh28, Lloh29
	.loh AdrpAdd	Lloh30, Lloh31
	.loh AdrpLdr	Lloh32, Lloh33
	.loh AdrpAdd	Lloh34, Lloh35
	.loh AdrpLdr	Lloh36, Lloh37
	.loh AdrpAdd	Lloh38, Lloh39
	.loh AdrpLdr	Lloh40, Lloh41
	.loh AdrpAdd	Lloh42, Lloh43
	.loh AdrpLdr	Lloh44, Lloh45
	.loh AdrpAdd	Lloh46, Lloh47
Lfunc_end0:
	.cfi_endproc
	.section	__TEXT,__gcc_except_tab
	.p2align	2, 0x0
GCC_except_table3:
Lexception0:
	.byte	255
	.byte	255
	.byte	1
	.uleb128 Lcst_end0-Lcst_begin0
Lcst_begin0:
	.uleb128 Lfunc_begin0-Lfunc_begin0
	.uleb128 Ltmp0-Lfunc_begin0
	.byte	0
	.byte	0
	.uleb128 Ltmp0-Lfunc_begin0
	.uleb128 Ltmp1-Ltmp0
	.uleb128 Ltmp2-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp21-Lfunc_begin0
	.uleb128 Ltmp22-Ltmp21
	.uleb128 Ltmp23-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp24-Lfunc_begin0
	.uleb128 Ltmp25-Ltmp24
	.uleb128 Ltmp32-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp33-Lfunc_begin0
	.uleb128 Ltmp34-Ltmp33
	.uleb128 Ltmp35-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp26-Lfunc_begin0
	.uleb128 Ltmp27-Ltmp26
	.uleb128 Ltmp32-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp27-Lfunc_begin0
	.uleb128 Ltmp30-Ltmp27
	.byte	0
	.byte	0
	.uleb128 Ltmp30-Lfunc_begin0
	.uleb128 Ltmp29-Ltmp30
	.uleb128 Ltmp32-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp29-Lfunc_begin0
	.uleb128 Ltmp3-Ltmp29
	.byte	0
	.byte	0
	.uleb128 Ltmp3-Lfunc_begin0
	.uleb128 Ltmp4-Ltmp3
	.uleb128 Ltmp5-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp4-Lfunc_begin0
	.uleb128 Ltmp6-Ltmp4
	.byte	0
	.byte	0
	.uleb128 Ltmp6-Lfunc_begin0
	.uleb128 Ltmp7-Ltmp6
	.uleb128 Ltmp8-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp9-Lfunc_begin0
	.uleb128 Ltmp10-Ltmp9
	.uleb128 Ltmp11-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp12-Lfunc_begin0
	.uleb128 Ltmp13-Ltmp12
	.uleb128 Ltmp14-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp15-Lfunc_begin0
	.uleb128 Ltmp16-Ltmp15
	.uleb128 Ltmp17-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp18-Lfunc_begin0
	.uleb128 Ltmp19-Ltmp18
	.uleb128 Ltmp20-Lfunc_begin0
	.byte	0
Lcst_end0:
	.p2align	2, 0x0

	.section	__TEXT,__text,regular,pure_instructions
	.private_extern	__ZN12rust_kernels4main17h0971da55851eba68E
	.globl	__ZN12rust_kernels4main17h0971da55851eba68E
	.p2align	2
__ZN12rust_kernels4main17h0971da55851eba68E:
Lfunc_begin1:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception1
	sub	sp, sp, #112
	.cfi_def_cfa_offset 112
	stp	x22, x21, [sp, #64]
	stp	x20, x19, [sp, #80]
	stp	x29, x30, [sp, #96]
	add	x29, sp, #96
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_remember_state
	add	x8, sp, #8
	bl	__ZN3std3env4args17hcf8cb98291c76d05E
Ltmp36:
	add	x8, sp, #40
	add	x0, sp, #8
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp37:
	ldr	x1, [sp, #40]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.eq	LBB4_6
	cbz	x1, LBB4_4
	ldr	x0, [sp, #48]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_4:
Ltmp39:
	add	x8, sp, #40
	add	x0, sp, #8
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp40:
	ldr	x1, [sp, #40]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.ne	LBB4_7
LBB4_6:
	mov	w19, #4096
	b	LBB4_27
LBB4_7:
	ldp	x0, x10, [sp, #48]
	cbz	x10, LBB4_12
	subs	x8, x10, #1
	b.ne	LBB4_13
	ldrb	w8, [x0]
	mov	w19, #4096
	cmp	w8, #43
	b.eq	LBB4_25
	cmp	w8, #45
	b.eq	LBB4_25
	mov	w8, #1
	mov	x9, x0
	b	LBB4_21
LBB4_12:
	mov	w19, #4096
	cbnz	x1, LBB4_26
	b	LBB4_27
LBB4_13:
	ldrb	w9, [x0]
	cmp	w9, #43
	b.ne	LBB4_20
	add	x9, x0, #1
	cmp	x10, #18
	b.lo	LBB4_21
LBB4_15:
	mov	x10, #0
	mov	w19, #4096
	mov	w11, #10
LBB4_16:
	cbz	x8, LBB4_72
	ldrb	w12, [x9], #1
	sub	w12, w12, #48
	cmp	w12, #9
	b.hi	LBB4_25
	umulh	x13, x10, x11
	cmp	xzr, x13
	cset	w13, ne
	add	x10, x10, x10, lsl #2
	lsl	x10, x10, #1
	adds	x10, x10, w12, uxtw
	cset	w12, hs
	tbnz	w13, #0, LBB4_25
	sub	x8, x8, #1
	tbz	w12, #0, LBB4_16
	b	LBB4_25
LBB4_20:
	mov	x9, x0
	mov	x8, x10
	cmp	x10, #17
	b.hs	LBB4_15
LBB4_21:
	mov	x19, #0
	mov	w10, #10
LBB4_22:
	ldrb	w11, [x9], #1
	sub	w11, w11, #48
	cmp	w11, #9
	b.hi	LBB4_24
	mul	x12, x19, x10
	add	x19, x12, w11, uxtw
	subs	x8, x8, #1
	b.ne	LBB4_22
	b	LBB4_25
LBB4_24:
	mov	w19, #4096
LBB4_25:
	cbz	x1, LBB4_27
LBB4_26:
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_27:
	ldr	x8, [sp, #16]
	ldr	x9, [sp, #32]
	subs	x9, x9, x8
	b.eq	LBB4_32
	mov	x10, #-6148914691236517206
	movk	x10, #43691
	umulh	x9, x9, x10
	lsr	x20, x9, #4
	add	x21, x8, #8
	b	LBB4_30
LBB4_29:
	add	x21, x21, #24
	subs	x20, x20, #1
	b.eq	LBB4_32
LBB4_30:
	ldur	x1, [x21, #-8]
	cbz	x1, LBB4_29
	ldr	x0, [x21]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB4_29
LBB4_32:
	ldr	x8, [sp, #24]
	cbz	x8, LBB4_34
	ldr	x0, [sp, #8]
	add	x8, x8, x8, lsl #1
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_34:
	add	x8, sp, #8
	bl	__ZN3std3env4args17hcf8cb98291c76d05E
Ltmp42:
	add	x8, sp, #40
	add	x0, sp, #8
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp43:
	ldr	x1, [sp, #40]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.eq	LBB4_51
	cbz	x1, LBB4_38
	ldr	x0, [sp, #48]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_38:
Ltmp44:
	add	x8, sp, #40
	add	x0, sp, #8
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp45:
	ldr	x1, [sp, #40]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.eq	LBB4_51
	cbz	x1, LBB4_42
	ldr	x0, [sp, #48]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_42:
Ltmp47:
	add	x8, sp, #40
	add	x0, sp, #8
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp48:
	mov	w20, #34464
	movk	w20, #1, lsl #16
	ldr	x1, [sp, #40]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.eq	LBB4_52
	ldp	x0, x10, [sp, #48]
	cbz	x10, LBB4_48
	subs	x8, x10, #1
	b.ne	LBB4_60
	ldrb	w8, [x0]
	cmp	w8, #43
	b.eq	LBB4_48
	cmp	w8, #45
	b.ne	LBB4_68
LBB4_48:
	mov	w20, #34464
	movk	w20, #1, lsl #16
LBB4_49:
	cbz	x1, LBB4_52
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB4_52
LBB4_51:
	mov	w20, #34464
	movk	w20, #1, lsl #16
LBB4_52:
	ldr	x8, [sp, #16]
	ldr	x9, [sp, #32]
	subs	x9, x9, x8
	b.eq	LBB4_57
	mov	x10, #-6148914691236517206
	movk	x10, #43691
	umulh	x9, x9, x10
	lsr	x21, x9, #4
	add	x22, x8, #8
	b	LBB4_55
LBB4_54:
	add	x22, x22, #24
	subs	x21, x21, #1
	b.eq	LBB4_57
LBB4_55:
	ldur	x1, [x22, #-8]
	cbz	x1, LBB4_54
	ldr	x0, [x22]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB4_54
LBB4_57:
	ldr	x8, [sp, #24]
	cbz	x8, LBB4_59
	ldr	x0, [sp, #8]
	add	x8, x8, x8, lsl #1
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_59:
Lloh48:
	adrp	x0, l_anon.e5f937a298947f43ee04ff650c15395e.10@PAGE
Lloh49:
	add	x0, x0, l_anon.e5f937a298947f43ee04ff650c15395e.10@PAGEOFF
Lloh50:
	adrp	x4, __ZN12rust_kernels14kernel_obvious17ha02a80708fbb71a5E@PAGE
Lloh51:
	add	x4, x4, __ZN12rust_kernels14kernel_obvious17ha02a80708fbb71a5E@PAGEOFF
	mov	w1, #12
	mov	x2, x19
	mov	x3, x20
	bl	__ZN12rust_kernels3run17h0f560483664fa09bE
Lloh52:
	adrp	x0, l_anon.e5f937a298947f43ee04ff650c15395e.11@PAGE
Lloh53:
	add	x0, x0, l_anon.e5f937a298947f43ee04ff650c15395e.11@PAGEOFF
Lloh54:
	adrp	x4, __ZN12rust_kernels13kernel_rebind17h842e7cdabfaa07e6E@PAGE
Lloh55:
	add	x4, x4, __ZN12rust_kernels13kernel_rebind17h842e7cdabfaa07e6E@PAGEOFF
	mov	w1, #11
	mov	x2, x19
	mov	x3, x20
	bl	__ZN12rust_kernels3run17h0f560483664fa09bE
Lloh56:
	adrp	x0, l_anon.e5f937a298947f43ee04ff650c15395e.12@PAGE
Lloh57:
	add	x0, x0, l_anon.e5f937a298947f43ee04ff650c15395e.12@PAGEOFF
Lloh58:
	adrp	x4, __ZN12rust_kernels14kernel_innerfn17h8bfca1a0dbfe6d64E@PAGE
Lloh59:
	add	x4, x4, __ZN12rust_kernels14kernel_innerfn17h8bfca1a0dbfe6d64E@PAGEOFF
	mov	w1, #12
	mov	x2, x19
	mov	x3, x20
	bl	__ZN12rust_kernels3run17h0f560483664fa09bE
	.cfi_def_cfa wsp, 112
	ldp	x29, x30, [sp, #96]
	ldp	x20, x19, [sp, #80]
	ldp	x22, x21, [sp, #64]
	add	sp, sp, #112
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	ret
LBB4_60:
	.cfi_restore_state
	ldrb	w9, [x0]
	cmp	w9, #43
	b.ne	LBB4_62
	add	x9, x0, #1
	cmp	x10, #18
	b.hs	LBB4_63
	b	LBB4_69
LBB4_62:
	mov	x9, x0
	mov	x8, x10
	cmp	x10, #17
	b.lo	LBB4_69
LBB4_63:
	mov	x20, #0
	mov	w10, #10
LBB4_64:
	cbz	x8, LBB4_49
	ldrb	w11, [x9], #1
	sub	w11, w11, #48
	cmp	w11, #9
	b.hi	LBB4_48
	umulh	x12, x20, x10
	cmp	xzr, x12
	cset	w12, ne
	add	x13, x20, x20, lsl #2
	lsl	x13, x13, #1
	adds	x20, x13, w11, uxtw
	cset	w11, hs
	tbnz	w12, #0, LBB4_48
	sub	x8, x8, #1
	tbz	w11, #0, LBB4_64
	b	LBB4_48
LBB4_68:
	mov	w8, #1
	mov	x9, x0
LBB4_69:
	mov	x20, #0
	mov	w10, #10
LBB4_70:
	ldrb	w11, [x9], #1
	sub	w11, w11, #48
	cmp	w11, #9
	b.hi	LBB4_48
	mul	x12, x20, x10
	add	x20, x12, w11, uxtw
	subs	x8, x8, #1
	b.ne	LBB4_70
	b	LBB4_49
LBB4_72:
	mov	x19, x10
	cbnz	x1, LBB4_26
	b	LBB4_27
LBB4_73:
Ltmp49:
	mov	x19, x0
	add	x0, sp, #8
	bl	__ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17he1b4bd5c88aa65f0E
	mov	x0, x19
	bl	__Unwind_Resume
LBB4_74:
Ltmp41:
	mov	x19, x0
	add	x0, sp, #8
	bl	__ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17he1b4bd5c88aa65f0E
	mov	x0, x19
	bl	__Unwind_Resume
LBB4_75:
Ltmp38:
	mov	x19, x0
	add	x0, sp, #8
	bl	__ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17he1b4bd5c88aa65f0E
	mov	x0, x19
	bl	__Unwind_Resume
LBB4_76:
Ltmp46:
	mov	x19, x0
	add	x0, sp, #8
	bl	__ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17he1b4bd5c88aa65f0E
	mov	x0, x19
	bl	__Unwind_Resume
	.loh AdrpAdd	Lloh58, Lloh59
	.loh AdrpAdd	Lloh56, Lloh57
	.loh AdrpAdd	Lloh54, Lloh55
	.loh AdrpAdd	Lloh52, Lloh53
	.loh AdrpAdd	Lloh50, Lloh51
	.loh AdrpAdd	Lloh48, Lloh49
Lfunc_end1:
	.cfi_endproc
	.section	__TEXT,__gcc_except_tab
	.p2align	2, 0x0
GCC_except_table4:
Lexception1:
	.byte	255
	.byte	255
	.byte	1
	.uleb128 Lcst_end1-Lcst_begin1
Lcst_begin1:
	.uleb128 Lfunc_begin1-Lfunc_begin1
	.uleb128 Ltmp36-Lfunc_begin1
	.byte	0
	.byte	0
	.uleb128 Ltmp36-Lfunc_begin1
	.uleb128 Ltmp37-Ltmp36
	.uleb128 Ltmp38-Lfunc_begin1
	.byte	0
	.uleb128 Ltmp39-Lfunc_begin1
	.uleb128 Ltmp40-Ltmp39
	.uleb128 Ltmp41-Lfunc_begin1
	.byte	0
	.uleb128 Ltmp40-Lfunc_begin1
	.uleb128 Ltmp42-Ltmp40
	.byte	0
	.byte	0
	.uleb128 Ltmp42-Lfunc_begin1
	.uleb128 Ltmp45-Ltmp42
	.uleb128 Ltmp46-Lfunc_begin1
	.byte	0
	.uleb128 Ltmp47-Lfunc_begin1
	.uleb128 Ltmp48-Ltmp47
	.uleb128 Ltmp49-Lfunc_begin1
	.byte	0
	.uleb128 Ltmp48-Lfunc_begin1
	.uleb128 Lfunc_end1-Ltmp48
	.byte	0
	.byte	0
Lcst_end1:
	.p2align	2, 0x0

	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__ZN12rust_kernels5inner17he7900cc6e13749c7E:
	.cfi_startproc
	stp	x20, x19, [sp, #-16]!
	.cfi_def_cfa_offset 16
	.cfi_offset w19, -8
	.cfi_offset w20, -16
	ldr	x8, [sp, #72]
	ldr	x9, [sp, #56]
	ldr	x10, [sp, #40]
	ldr	x11, [sp, #24]
	cmp	x3, x1
	csel	x12, x3, x1, lo
	cmp	x5, x12
	csel	x12, x5, x12, lo
	cmp	x7, x12
	csel	x12, x7, x12, lo
	cmp	x11, x12
	csel	x11, x11, x12, lo
	cmp	x10, x11
	csel	x10, x10, x11, lo
	cmp	x9, x10
	csel	x9, x9, x10, lo
	cmp	x8, x9
	csel	x12, x8, x9, lo
	cbz	x12, LBB5_8
	ldr	x8, [sp, #64]
	ldr	x9, [sp, #48]
	ldr	x10, [sp, #32]
	ldr	x11, [sp, #16]
	cmp	x12, #4
	b.hs	LBB5_3
	mov	x13, #0
	b	LBB5_6
LBB5_3:
	and	x13, x12, #0xfffffffffffffffc
	add	x14, x0, #16
	add	x15, x8, #16
	add	x16, x11, #16
	add	x17, x9, #16
	add	x1, x10, #16
	add	x3, x2, #16
	add	x5, x4, #16
	add	x7, x6, #16
	mov	x19, x13
LBB5_4:
	ldp	q0, q1, [x14, #-16]
	ldp	q2, q3, [x5, #-16]
	ldp	q4, q5, [x7, #-16]
	ldp	q6, q7, [x16, #-16]
	ldp	q16, q17, [x1, #-16]
	add.2d	v6, v16, v6
	add.2d	v7, v17, v7
	add.2d	v0, v0, v2
	add.2d	v0, v6, v0
	add.2d	v1, v1, v3
	add.2d	v1, v7, v1
	add.2d	v0, v0, v4
	add.2d	v1, v1, v5
	stp	q0, q1, [x14, #-16]
	ldp	q0, q1, [x3, #-16]
	ldp	q2, q3, [x17, #-16]
	ldp	q4, q5, [x15, #-16]
	add.2d	v0, v0, v2
	add.2d	v0, v6, v0
	add.2d	v1, v1, v3
	add.2d	v1, v7, v1
	add.2d	v0, v0, v4
	add	x14, x14, #32
	add.2d	v1, v1, v5
	add	x15, x15, #32
	add	x16, x16, #32
	add	x17, x17, #32
	add	x1, x1, #32
	stp	q0, q1, [x3, #-16]
	add	x3, x3, #32
	add	x5, x5, #32
	add	x7, x7, #32
	subs	x19, x19, #4
	b.ne	LBB5_4
	cmp	x12, x13
	b.eq	LBB5_8
LBB5_6:
	mov	x14, #0
	sub	x12, x12, x13
	lsl	x1, x13, #3
	add	x13, x0, x1
	add	x15, x4, x1
	add	x16, x6, x1
	add	x11, x11, x1
	add	x10, x10, x1
	add	x17, x2, x1
	add	x9, x9, x1
	add	x8, x8, x1
LBB5_7:
	ldr	x0, [x13, x14, lsl #3]
	ldr	x1, [x15, x14, lsl #3]
	ldr	x2, [x16, x14, lsl #3]
	ldr	x3, [x11, x14, lsl #3]
	ldr	x4, [x10, x14, lsl #3]
	add	x3, x4, x3
	add	x0, x0, x1
	add	x0, x3, x0
	add	x0, x0, x2
	str	x0, [x13, x14, lsl #3]
	ldr	x0, [x17, x14, lsl #3]
	ldr	x1, [x9, x14, lsl #3]
	ldr	x2, [x8, x14, lsl #3]
	add	x0, x0, x1
	add	x0, x3, x0
	add	x0, x0, x2
	str	x0, [x17, x14, lsl #3]
	add	x14, x14, #1
	cmp	x12, x14
	b.ne	LBB5_7
LBB5_8:
	ldp	x20, x19, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w19
	.cfi_restore w20
	ret
	.cfi_endproc

	.private_extern	__ZN3std2rt10lang_start17h3ad8f29af2a05baeE
	.globl	__ZN3std2rt10lang_start17h3ad8f29af2a05baeE
	.p2align	2
__ZN3std2rt10lang_start17h3ad8f29af2a05baeE:
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
Lloh60:
	adrp	x1, l_anon.e5f937a298947f43ee04ff650c15395e.13@PAGE
Lloh61:
	add	x1, x1, l_anon.e5f937a298947f43ee04ff650c15395e.13@PAGEOFF
	add	x0, sp, #8
	bl	__ZN3std2rt19lang_start_internal17hd700ba983d3377dcE
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.loh AdrpAdd	Lloh60, Lloh61
	.cfi_endproc

	.p2align	2
__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hf939136bc8cf78bdE:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x0, [x0]
	bl	__ZN3std3sys9backtrace28__rust_begin_short_backtrace17h7c311f337938dfb5E
	mov	w0, #0
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__ZN3std3sys9backtrace28__rust_begin_short_backtrace17h7c311f337938dfb5E:
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
__ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h18cb492d659b6813E:
	.cfi_startproc
	mov	x2, x1
	ldp	x8, x1, [x0]
	mov	x0, x8
	b	__ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h1e7192c885a6ee43E
	.cfi_endproc

	.p2align	2
__ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h53a59d359e201cd3E:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x0, [x0]
	bl	__ZN3std3sys9backtrace28__rust_begin_short_backtrace17h7c311f337938dfb5E
	mov	w0, #0
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17he1b4bd5c88aa65f0E:
	.cfi_startproc
	stp	x22, x21, [sp, #-48]!
	.cfi_def_cfa_offset 48
	stp	x20, x19, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_remember_state
	mov	x19, x0
	ldr	x8, [x0, #8]
	ldr	x9, [x0, #24]
	subs	x9, x9, x8
	b.eq	LBB11_5
	mov	x10, #-6148914691236517206
	movk	x10, #43691
	umulh	x9, x9, x10
	lsr	x20, x9, #4
	add	x21, x8, #8
	b	LBB11_3
LBB11_2:
	add	x21, x21, #24
	subs	x20, x20, #1
	b.eq	LBB11_5
LBB11_3:
	ldur	x1, [x21, #-8]
	cbz	x1, LBB11_2
	ldr	x0, [x21]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB11_2
LBB11_5:
	ldr	x8, [x19, #16]
	cbz	x8, LBB11_7
	ldr	x0, [x19]
	add	x8, x8, x8, lsl #1
	lsl	x1, x8, #3
	mov	w2, #8
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	ldp	x22, x21, [sp], #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	b	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB11_7:
	.cfi_restore_state
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	ldp	x22, x21, [sp], #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	ret
	.cfi_endproc

	.p2align	2
__ZN4core3ptr39drop_in_place$LT$rust_kernels..Cols$GT$17h1f70d29370f747a4E:
	.cfi_startproc
	stp	x20, x19, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	mov	x19, x0
	ldr	x8, [x0]
	cbz	x8, LBB12_2
	ldr	x0, [x19, #8]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_2:
	ldr	x8, [x19, #24]
	cbz	x8, LBB12_4
	ldr	x0, [x19, #32]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_4:
	ldr	x8, [x19, #48]
	cbz	x8, LBB12_6
	ldr	x0, [x19, #56]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_6:
	ldr	x8, [x19, #72]
	cbz	x8, LBB12_8
	ldr	x0, [x19, #80]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_8:
	ldr	x8, [x19, #96]
	cbz	x8, LBB12_10
	ldr	x0, [x19, #104]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_10:
	ldr	x8, [x19, #120]
	cbz	x8, LBB12_12
	ldr	x0, [x19, #128]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_12:
	ldr	x8, [x19, #144]
	cbz	x8, LBB12_14
	ldr	x0, [x19, #152]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_14:
	ldr	x8, [x19, #168]
	cbz	x8, LBB12_16
	ldr	x0, [x19, #176]
	lsl	x1, x8, #3
	mov	w2, #8
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	b	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB12_16:
	.cfi_restore_state
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
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
Lloh62:
	adrp	x8, __ZN12rust_kernels4main17h0971da55851eba68E@PAGE
Lloh63:
	add	x8, x8, __ZN12rust_kernels4main17h0971da55851eba68E@PAGEOFF
	str	x8, [sp, #8]
Lloh64:
	adrp	x1, l_anon.e5f937a298947f43ee04ff650c15395e.13@PAGE
Lloh65:
	add	x1, x1, l_anon.e5f937a298947f43ee04ff650c15395e.13@PAGEOFF
	add	x0, sp, #8
	mov	w4, #0
	bl	__ZN3std2rt19lang_start_internal17hd700ba983d3377dcE
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	ret
	.loh AdrpAdd	Lloh64, Lloh65
	.loh AdrpAdd	Lloh62, Lloh63
	.cfi_endproc

	.section	__TEXT,__cstring,cstring_literals
l_anon.e5f937a298947f43ee04ff650c15395e.0:
	.asciz	"rust_kernels.rs"

	.section	__TEXT,__literal4,4byte_literals
l_anon.e5f937a298947f43ee04ff650c15395e.1:
	.ascii	": n="

	.section	__TEXT,__const
l_anon.e5f937a298947f43ee04ff650c15395e.2:
	.ascii	" k="

l_anon.e5f937a298947f43ee04ff650c15395e.3:
	.ascii	" ns/elem="

l_anon.e5f937a298947f43ee04ff650c15395e.4:
	.ascii	" checksum="

l_anon.e5f937a298947f43ee04ff650c15395e.5:
	.byte	10

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.e5f937a298947f43ee04ff650c15395e.6:
	.quad	1
	.space	8
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.1
	.asciz	"\004\000\000\000\000\000\000"
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.2
	.asciz	"\003\000\000\000\000\000\000"
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.3
	.asciz	"\t\000\000\000\000\000\000"
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.4
	.asciz	"\n\000\000\000\000\000\000"
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.5
	.asciz	"\001\000\000\000\000\000\000"

	.section	__TEXT,__const
	.p2align	3, 0x0
l_anon.e5f937a298947f43ee04ff650c15395e.7:
	.asciz	"\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000 \000\000\340\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\001\000\000\000\000\000\000\000 \000\000\340\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000 \000\000\340\000\000\000\000\000\000\003\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\003\000\000\000\000\000\000\000 \000\000\360\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\004\000\000\000\000\000\000\000 \000\000\340\000\000\000"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.e5f937a298947f43ee04ff650c15395e.8:
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.0
	.asciz	"\017\000\000\000\000\000\000\000D\000\000\000&\000\000"

	.p2align	3, 0x0
l_anon.e5f937a298947f43ee04ff650c15395e.9:
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.0
	.asciz	"\017\000\000\000\000\000\000\000D\000\000\000/\000\000"

	.section	__TEXT,__const
l_anon.e5f937a298947f43ee04ff650c15395e.10:
	.ascii	"rust-obvious"

l_anon.e5f937a298947f43ee04ff650c15395e.11:
	.ascii	"rust-rebind"

l_anon.e5f937a298947f43ee04ff650c15395e.12:
	.ascii	"rust-innerfn"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.e5f937a298947f43ee04ff650c15395e.13:
	.asciz	"\000\000\000\000\000\000\000\000\b\000\000\000\000\000\000\000\b\000\000\000\000\000\000"
	.quad	__ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h53a59d359e201cd3E
	.quad	__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hf939136bc8cf78bdE
	.quad	__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hf939136bc8cf78bdE

	.section	__TEXT,__cstring,cstring_literals
l_anon.e5f937a298947f43ee04ff650c15395e.14:
	.asciz	"<local-home>/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.e5f937a298947f43ee04ff650c15395e.15:
	.quad	l_anon.e5f937a298947f43ee04ff650c15395e.14
	.asciz	"}\000\000\000\000\000\000\000\353\007\000\000\t\000\000"

.subsections_via_symbols

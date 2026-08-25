	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.private_extern	__ZN3std2rt10lang_start17hf0285bbbfe2a1d69E
	.globl	__ZN3std2rt10lang_start17hf0285bbbfe2a1d69E
	.p2align	2
__ZN3std2rt10lang_start17hf0285bbbfe2a1d69E:
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
	adrp	x1, l_anon.8883933be1ddab77f0e070c5ffe28e0c.0@PAGE
Lloh1:
	add	x1, x1, l_anon.8883933be1ddab77f0e070c5ffe28e0c.0@PAGEOFF
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
__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h7ac6eab5420d5d32E:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x0, [x0]
	bl	__ZN3std3sys9backtrace28__rust_begin_short_backtrace17hece1e6645d5dfacbE
	mov	w0, #0
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__ZN3std3sys9backtrace28__rust_begin_short_backtrace17hece1e6645d5dfacbE:
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
__ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h59c06508251e6ee3E:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x0, [x0]
	bl	__ZN3std3sys9backtrace28__rust_begin_short_backtrace17hece1e6645d5dfacbE
	mov	w0, #0
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__ZN4core3ptr33drop_in_place$LT$rust16..Wide$GT$17hf9bad0f52cc3b8d9E:
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
	cbz	x8, LBB4_2
	ldr	x0, [x19, #8]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_2:
	ldr	x8, [x19, #24]
	cbz	x8, LBB4_4
	ldr	x0, [x19, #32]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_4:
	ldr	x8, [x19, #48]
	cbz	x8, LBB4_6
	ldr	x0, [x19, #56]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_6:
	ldr	x8, [x19, #72]
	cbz	x8, LBB4_8
	ldr	x0, [x19, #80]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_8:
	ldr	x8, [x19, #96]
	cbz	x8, LBB4_10
	ldr	x0, [x19, #104]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_10:
	ldr	x8, [x19, #120]
	cbz	x8, LBB4_12
	ldr	x0, [x19, #128]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_12:
	ldr	x8, [x19, #144]
	cbz	x8, LBB4_14
	ldr	x0, [x19, #152]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_14:
	ldr	x8, [x19, #168]
	cbz	x8, LBB4_16
	ldr	x0, [x19, #176]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_16:
	ldr	x8, [x19, #192]
	cbz	x8, LBB4_18
	ldr	x0, [x19, #200]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_18:
	ldr	x8, [x19, #216]
	cbz	x8, LBB4_20
	ldr	x0, [x19, #224]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_20:
	ldr	x8, [x19, #240]
	cbz	x8, LBB4_22
	ldr	x0, [x19, #248]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_22:
	ldr	x8, [x19, #264]
	cbz	x8, LBB4_24
	ldr	x0, [x19, #272]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_24:
	ldr	x8, [x19, #288]
	cbz	x8, LBB4_26
	ldr	x0, [x19, #296]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_26:
	ldr	x8, [x19, #312]
	cbz	x8, LBB4_28
	ldr	x0, [x19, #320]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_28:
	ldr	x8, [x19, #336]
	cbz	x8, LBB4_30
	ldr	x0, [x19, #344]
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB4_30:
	ldr	x8, [x19, #360]
	cbz	x8, LBB4_32
	ldr	x0, [x19, #368]
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
LBB4_32:
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

	.p2align	2
__ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hd9e9aabfcd1b7715E:
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
	b.eq	LBB5_5
	mov	x10, #-6148914691236517206
	movk	x10, #43691
	umulh	x9, x9, x10
	lsr	x20, x9, #4
	add	x21, x8, #8
	b	LBB5_3
LBB5_2:
	add	x21, x21, #24
	subs	x20, x20, #1
	b.eq	LBB5_5
LBB5_3:
	ldur	x1, [x21, #-8]
	cbz	x1, LBB5_2
	ldr	x0, [x21]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB5_2
LBB5_5:
	ldr	x8, [x19, #16]
	cbz	x8, LBB5_7
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
LBB5_7:
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
__ZN6rust1614kernel_obvious17he2f3712ce068d2d4E:
	.cfi_startproc
	sub	sp, sp, #416
	.cfi_def_cfa_offset 416
	stp	x28, x27, [sp, #320]
	stp	x26, x25, [sp, #336]
	stp	x24, x23, [sp, #352]
	stp	x22, x21, [sp, #368]
	stp	x20, x19, [sp, #384]
	stp	x29, x30, [sp, #400]
	add	x29, sp, #400
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
	ldr	x8, [x0, #16]
	ldr	x9, [x0, #40]
	ldr	x10, [x0, #64]
	ldr	x11, [x0, #88]
	ldr	x12, [x0, #112]
	ldr	x13, [x0, #136]
	ldr	x14, [x0, #160]
	ldr	x15, [x0, #184]
	ldr	x16, [x0, #208]
	ldr	x17, [x0, #232]
	ldr	x3, [x0, #256]
	ldr	x1, [x0, #280]
	ldr	x4, [x0, #304]
	ldr	x5, [x0, #328]
	ldr	x2, [x0, #352]
	ldr	x6, [x0, #376]
	cmp	x6, x2
	stp	x6, x2, [x29, #-120]
	csel	x2, x6, x2, lo
	cmp	x2, x5
	stp	x5, x4, [x29, #-104]
	csel	x2, x2, x5, lo
	cmp	x2, x4
	csel	x2, x2, x4, lo
	cmp	x2, x1
	csel	x2, x2, x1, lo
	cmp	x2, x3
	csel	x2, x2, x3, lo
	cmp	x2, x17
	csel	x17, x2, x17, lo
	cmp	x17, x16
	csel	x16, x17, x16, lo
	cmp	x16, x15
	csel	x15, x16, x15, lo
	cmp	x15, x14
	csel	x14, x15, x14, lo
	cmp	x14, x13
	csel	x13, x14, x13, lo
	cmp	x13, x12
	csel	x12, x13, x12, lo
	cmp	x12, x11
	csel	x11, x12, x11, lo
	cmp	x11, x10
	csel	x10, x11, x10, lo
	cmp	x10, x9
	csel	x9, x10, x9, lo
	cmp	x9, x8
	csel	x26, x9, x8, lo
	cbz	x26, LBB6_66
	ldr	x12, [x0, #8]
	ldr	x13, [x0, #104]
	ldr	x14, [x0, #128]
	ldr	x15, [x0, #152]
	ldr	x16, [x0, #32]
	ldr	x17, [x0, #176]
	ldr	x2, [x0, #200]
	ldr	x3, [x0, #224]
	ldr	x4, [x0, #56]
	ldr	x5, [x0, #248]
	ldp	x11, x10, [x29, #-104]
	cmp	x10, x1
	csel	x8, x10, x1, lo
	ldr	x6, [x0, #272]
	ldr	x7, [x0, #296]
	cmp	x8, x11
	csel	x8, x8, x11, lo
	ldp	x25, x23, [x29, #-120]
	cmp	x8, x23
	csel	x8, x8, x23, lo
	ldr	x19, [x0, #80]
	ldr	x20, [x0, #320]
	cmp	x8, x25
	csel	x9, x8, x25, lo
	sub	x8, x26, #1
	ldr	x21, [x0, #344]
	cmp	x9, x8
	csel	x9, x9, x8, lo
	ldr	x22, [x0, #368]
	cmp	x9, #39
	b.hi	LBB6_3
	mov	x0, #0
	mov	w24, #1
	b	LBB6_59
LBB6_3:
	str	x9, [sp]
	stp	x26, x1, [x29, #-184]
	mov	x0, #0
	cmp	x10, x1
	csel	x9, x10, x1, lo
	cmp	x9, x11
	csel	x9, x9, x11, lo
	cmp	x9, x23
	csel	x9, x9, x23, lo
	cmp	x9, x25
	csel	x9, x9, x25, lo
	cmp	x9, x8
	csel	x8, x9, x8, lo
	lsl	x8, x8, #3
	add	x8, x8, #8
	add	x24, x12, x8
	add	x26, x16, x8
	add	x30, x4, x8
	cmp	x12, x30
	ccmp	x4, x24, #2, lo
	cset	w10, lo
	add	x27, x19, x8
	cmp	x12, x27
	ccmp	x19, x24, #2, lo
	cset	w9, lo
	stp	w9, w10, [x29, #-192]
	add	x10, x13, x8
	cmp	x12, x10
	ccmp	x13, x24, #2, lo
	cset	w9, lo
	stur	w9, [x29, #-196]
	add	x11, x14, x8
	cmp	x12, x11
	ccmp	x14, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #200]
	add	x1, x15, x8
	cmp	x12, x1
	ccmp	x15, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #196]
	add	x23, x17, x8
	cmp	x12, x23
	ccmp	x17, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #192]
	add	x9, x2, x8
	stur	x9, [x29, #-160]
	cmp	x12, x9
	ccmp	x2, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #188]
	add	x9, x3, x8
	stur	x9, [x29, #-168]
	cmp	x12, x9
	ccmp	x3, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #184]
	add	x25, x5, x8
	cmp	x12, x25
	str	x25, [sp, #72]
	ccmp	x5, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #180]
	add	x9, x6, x8
	stur	x9, [x29, #-128]
	cmp	x12, x9
	ccmp	x6, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #176]
	add	x9, x7, x8
	stur	x9, [x29, #-136]
	cmp	x12, x9
	ccmp	x7, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #172]
	add	x9, x20, x8
	stur	x9, [x29, #-144]
	cmp	x12, x9
	ccmp	x20, x24, #2, lo
	cset	w9, lo
	str	w9, [sp, #168]
	add	x9, x21, x8
	stur	x9, [x29, #-152]
	cmp	x12, x9
	ccmp	x21, x24, #2, lo
	cset	w9, lo
	add	x8, x22, x8
	str	x8, [sp, #80]
	cmp	x12, x8
	ccmp	x22, x24, #2, lo
	cset	w28, lo
	stp	w28, w9, [sp, #160]
	cmp	x16, x30
	ccmp	x4, x26, #2, lo
	cset	w8, lo
	cmp	x16, x27
	ccmp	x19, x26, #2, lo
	cset	w28, lo
	stp	w28, w8, [sp, #152]
	mov	x8, x10
	cmp	x16, x10
	ccmp	x13, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #148]
	mov	x9, x11
	stp	x1, x11, [sp, #56]
	cmp	x16, x11
	ccmp	x14, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #144]
	mov	x10, x1
	cmp	x16, x1
	ccmp	x15, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #140]
	mov	x11, x23
	str	x23, [sp, #48]
	cmp	x16, x23
	ccmp	x17, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #136]
	ldur	x1, [x29, #-160]
	cmp	x16, x1
	ccmp	x2, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #132]
	ldur	x23, [x29, #-168]
	cmp	x16, x23
	ccmp	x3, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #128]
	cmp	x16, x25
	ccmp	x5, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #124]
	ldur	x25, [x29, #-128]
	cmp	x16, x25
	ccmp	x6, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #120]
	ldur	x25, [x29, #-136]
	cmp	x16, x25
	ccmp	x7, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #116]
	ldur	x25, [x29, #-144]
	cmp	x16, x25
	ccmp	x20, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #112]
	ldur	x25, [x29, #-152]
	cmp	x16, x25
	ccmp	x21, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #108]
	ldr	x25, [sp, #80]
	cmp	x16, x25
	ccmp	x22, x26, #2, lo
	cset	w28, lo
	str	w28, [sp, #104]
	cmp	x4, x27
	ccmp	x19, x30, #2, lo
	cset	w28, lo
	str	w28, [sp, #100]
	cmp	x4, x8
	ccmp	x13, x30, #2, lo
	cset	w28, lo
	str	w28, [sp, #96]
	cmp	x4, x9
	ccmp	x14, x30, #2, lo
	cset	w9, lo
	cmp	x4, x10
	ccmp	x15, x30, #2, lo
	cset	w28, lo
	stp	w28, w9, [sp, #88]
	cmp	x4, x11
	ccmp	x17, x30, #2, lo
	cset	w9, lo
	cmp	x4, x1
	ccmp	x2, x30, #2, lo
	cset	w28, lo
	stp	w28, w9, [sp, #40]
	cmp	x4, x23
	ccmp	x3, x30, #2, lo
	cset	w1, lo
	ldr	x9, [sp, #72]
	cmp	x4, x9
	ccmp	x5, x30, #2, lo
	cset	w28, lo
	stp	w28, w1, [sp, #32]
	ldur	x10, [x29, #-128]
	cmp	x4, x10
	ccmp	x6, x30, #2, lo
	cset	w28, lo
	str	w28, [sp, #28]
	ldur	x11, [x29, #-136]
	cmp	x4, x11
	ccmp	x7, x30, #2, lo
	cset	w28, lo
	str	w28, [sp, #24]
	ldur	x1, [x29, #-144]
	cmp	x4, x1
	ccmp	x20, x30, #2, lo
	cset	w28, lo
	str	w28, [sp, #20]
	ldur	x23, [x29, #-152]
	cmp	x4, x23
	ccmp	x21, x30, #2, lo
	cset	w28, lo
	str	w28, [sp, #16]
	cmp	x4, x25
	ccmp	x22, x30, #2, lo
	cset	w28, lo
	cmp	x19, x8
	ccmp	x13, x27, #2, lo
	cset	w8, lo
	stp	w8, w28, [sp, #8]
	ldr	x8, [sp, #64]
	cmp	x19, x8
	ccmp	x14, x27, #2, lo
	cset	w8, lo
	str	w8, [sp, #64]
	ldp	x28, x8, [sp, #48]
	cmp	x19, x8
	ccmp	x15, x27, #2, lo
	cset	w8, lo
	cmp	x19, x28
	ccmp	x17, x27, #2, lo
	cset	w28, lo
	ldur	x30, [x29, #-160]
	cmp	x19, x30
	ccmp	x2, x27, #2, lo
	cset	w30, lo
	stur	w30, [x29, #-160]
	ldur	x30, [x29, #-168]
	cmp	x19, x30
	ccmp	x3, x27, #2, lo
	cset	w30, lo
	stur	w30, [x29, #-168]
	cmp	x19, x9
	ccmp	x5, x27, #2, lo
	cset	w9, lo
	str	w9, [sp, #72]
	cmp	x19, x10
	ccmp	x6, x27, #2, lo
	cset	w30, lo
	cmp	x19, x11
	ccmp	x7, x27, #2, lo
	cset	w9, lo
	cmp	x19, x1
	ccmp	x20, x27, #2, lo
	cset	w10, lo
	cmp	x19, x23
	ccmp	x21, x27, #2, lo
	cset	w11, lo
	cmp	x19, x25
	ccmp	x22, x27, #2, lo
	cset	w25, lo
	cmp	x16, x24
	ccmp	x12, x26, #2, lo
	mov	w24, #1
	b.lo	LBB6_71
	ldur	w1, [x29, #-188]
	tbnz	w1, #0, LBB6_71
	ldur	w1, [x29, #-192]
	tbnz	w1, #0, LBB6_71
	ldur	w1, [x29, #-196]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #200]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #196]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #192]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #188]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #184]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #180]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #176]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #172]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #168]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #164]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #160]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #156]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #152]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #148]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #144]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #140]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #136]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #132]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #128]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #124]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #120]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #116]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #112]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #108]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #104]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #100]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #96]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #92]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #88]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #44]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #40]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #36]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #32]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #28]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #24]
	tbnz	w1, #0, LBB6_71
	ldr	w1, [sp, #20]
	tbnz	w1, #0, LBB6_71
	mov	x23, x28
	mov	x28, x10
	ldr	w10, [sp, #16]
	tbnz	w10, #0, LBB6_71
	ldr	w10, [sp, #12]
	tbnz	w10, #0, LBB6_71
	ldr	w10, [sp, #8]
	tbnz	w10, #0, LBB6_71
	ldr	w10, [sp, #64]
	tbnz	w10, #0, LBB6_71
	tbnz	w8, #0, LBB6_71
	tbnz	w23, #0, LBB6_71
	ldur	w8, [x29, #-160]
	tbnz	w8, #0, LBB6_71
	ldur	x1, [x29, #-176]
	ldur	w8, [x29, #-168]
	tbnz	w8, #0, LBB6_72
	ldur	x26, [x29, #-184]
	ldr	w8, [sp, #72]
	tbnz	w8, #0, LBB6_70
	mov	x8, x25
	ldur	x25, [x29, #-120]
	tbnz	w30, #0, LBB6_69
	ldur	x23, [x29, #-112]
	tbnz	w9, #0, LBB6_68
	tbnz	w28, #0, LBB6_68
	mov	x27, x11
	ldur	x11, [x29, #-104]
	tbnz	w27, #0, LBB6_67
	ldur	x10, [x29, #-96]
	tbnz	w8, #0, LBB6_59
	mov	x8, #0
	ldr	x9, [sp]
	and	x0, x9, #0xffffffffffffffe
	orr	x24, x9, #0x1
	lsl	x9, x9, #3
	and	x9, x9, #0xfffffffffffffff0
LBB6_58:
	ldr	q0, [x12, x8]
	ldr	q1, [x13, x8]
	add.2d	v0, v1, v0
	ldr	q1, [x14, x8]
	ldr	q2, [x15, x8]
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	str	q0, [x12, x8]
	ldr	q0, [x16, x8]
	ldr	q1, [x17, x8]
	add.2d	v0, v1, v0
	ldr	q1, [x2, x8]
	ldr	q2, [x3, x8]
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	str	q0, [x16, x8]
	ldr	q0, [x4, x8]
	ldr	q1, [x5, x8]
	add.2d	v0, v1, v0
	ldr	q1, [x6, x8]
	ldr	q2, [x7, x8]
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	str	q0, [x4, x8]
	ldr	q0, [x19, x8]
	ldr	q1, [x20, x8]
	add.2d	v0, v1, v0
	ldr	q1, [x21, x8]
	ldr	q2, [x22, x8]
	add.2d	v1, v1, v2
	add.2d	v0, v0, v1
	str	q0, [x19, x8]
	add	x8, x8, #16
	cmp	x9, x8
	b.ne	LBB6_58
LBB6_59:
	add	x8, x1, #1
	add	x9, x10, #1
	add	x10, x11, #1
	add	x11, x23, #1
	add	x30, x25, #1
	add	x23, x26, #1
LBB6_60:
	ldr	x25, [x12, x0, lsl #3]
	ldr	x26, [x13, x0, lsl #3]
	add	x25, x26, x25
	ldr	x26, [x14, x0, lsl #3]
	ldr	x27, [x15, x0, lsl #3]
	add	x26, x26, x27
	add	x25, x25, x26
	str	x25, [x12, x0, lsl #3]
	ldr	x25, [x16, x0, lsl #3]
	ldr	x26, [x17, x0, lsl #3]
	add	x26, x26, x25
	ldr	x27, [x2, x0, lsl #3]
	ldr	x28, [x3, x0, lsl #3]
	mov	x25, x24
	add	x24, x27, x28
	add	x24, x26, x24
	str	x24, [x16, x0, lsl #3]
	cmp	x8, x25
	b.eq	LBB6_73
	cmp	x9, x25
	b.eq	LBB6_74
	ldr	x24, [x4, x0, lsl #3]
	ldr	x26, [x5, x0, lsl #3]
	ldr	x27, [x6, x0, lsl #3]
	ldr	x28, [x7, x0, lsl #3]
	add	x24, x26, x24
	add	x26, x27, x28
	add	x24, x24, x26
	str	x24, [x4, x0, lsl #3]
	cmp	x10, x25
	b.eq	LBB6_75
	cmp	x11, x25
	b.eq	LBB6_76
	cmp	x30, x25
	b.eq	LBB6_77
	ldr	x24, [x19, x0, lsl #3]
	ldr	x26, [x20, x0, lsl #3]
	add	x24, x26, x24
	ldr	x26, [x21, x0, lsl #3]
	ldr	x27, [x22, x0, lsl #3]
	add	x26, x26, x27
	add	x24, x24, x26
	str	x24, [x19, x0, lsl #3]
	add	x24, x25, #1
	mov	x0, x25
	cmp	x23, x24
	b.ne	LBB6_60
LBB6_66:
	.cfi_def_cfa wsp, 416
	ldp	x29, x30, [sp, #400]
	ldp	x20, x19, [sp, #384]
	ldp	x22, x21, [sp, #368]
	ldp	x24, x23, [sp, #352]
	ldp	x26, x25, [sp, #336]
	ldp	x28, x27, [sp, #320]
	add	sp, sp, #416
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
LBB6_67:
	.cfi_restore_state
	ldur	x10, [x29, #-96]
	b	LBB6_59
LBB6_68:
	ldp	x11, x10, [x29, #-104]
	b	LBB6_59
LBB6_69:
	ldp	x11, x10, [x29, #-104]
	ldur	x23, [x29, #-112]
	b	LBB6_59
LBB6_70:
	ldp	x11, x10, [x29, #-104]
	ldp	x25, x23, [x29, #-120]
	b	LBB6_59
LBB6_71:
	ldur	x1, [x29, #-176]
LBB6_72:
	ldp	x11, x10, [x29, #-104]
	ldp	x25, x23, [x29, #-120]
	ldur	x26, [x29, #-184]
	b	LBB6_59
LBB6_73:
Lloh2:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.4@PAGE
Lloh3:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.4@PAGEOFF
	bl	__ZN4core9panicking18panic_bounds_check17h8935b4e0a545757eE
LBB6_74:
Lloh4:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.5@PAGE
Lloh5:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.5@PAGEOFF
	ldur	x1, [x29, #-96]
	bl	__ZN4core9panicking18panic_bounds_check17h8935b4e0a545757eE
LBB6_75:
Lloh6:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.6@PAGE
Lloh7:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.6@PAGEOFF
	ldur	x1, [x29, #-104]
	bl	__ZN4core9panicking18panic_bounds_check17h8935b4e0a545757eE
LBB6_76:
Lloh8:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.7@PAGE
Lloh9:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.7@PAGEOFF
	ldur	x1, [x29, #-112]
	bl	__ZN4core9panicking18panic_bounds_check17h8935b4e0a545757eE
LBB6_77:
Lloh10:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.8@PAGE
Lloh11:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.8@PAGEOFF
	ldur	x1, [x29, #-120]
	bl	__ZN4core9panicking18panic_bounds_check17h8935b4e0a545757eE
	.loh AdrpAdd	Lloh2, Lloh3
	.loh AdrpAdd	Lloh4, Lloh5
	.loh AdrpAdd	Lloh6, Lloh7
	.loh AdrpAdd	Lloh8, Lloh9
	.loh AdrpAdd	Lloh10, Lloh11
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI7_0:
	.quad	0
	.quad	1
	.section	__TEXT,__text,regular,pure_instructions
	.private_extern	__ZN6rust164main17hdcfe507bfc2afffeE
	.globl	__ZN6rust164main17hdcfe507bfc2afffeE
	.p2align	2
__ZN6rust164main17hdcfe507bfc2afffeE:
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
	sub	sp, sp, #752
	add	x8, sp, #216
	bl	__ZN3std3env4args17hcf8cb98291c76d05E
Ltmp0:
	sub	x8, x29, #152
	add	x0, sp, #216
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp1:
	ldur	x1, [x29, #-152]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.eq	LBB7_6
	cbz	x1, LBB7_4
	ldur	x0, [x29, #-144]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_4:
Ltmp3:
	sub	x8, x29, #152
	add	x0, sp, #216
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp4:
	ldur	x1, [x29, #-152]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.ne	LBB7_7
LBB7_6:
	mov	w26, #4096
	b	LBB7_27
LBB7_7:
	ldp	x0, x10, [x29, #-144]
	cbz	x10, LBB7_12
	subs	x8, x10, #1
	b.ne	LBB7_13
	ldrb	w8, [x0]
	mov	w26, #4096
	cmp	w8, #43
	b.eq	LBB7_25
	cmp	w8, #45
	b.eq	LBB7_25
	mov	w8, #1
	mov	x9, x0
	b	LBB7_21
LBB7_12:
	mov	w26, #4096
	cbnz	x1, LBB7_26
	b	LBB7_27
LBB7_13:
	ldrb	w9, [x0]
	cmp	w9, #43
	b.ne	LBB7_20
	add	x9, x0, #1
	cmp	x10, #18
	b.lo	LBB7_21
LBB7_15:
	mov	x10, #0
	mov	w26, #4096
	mov	w11, #10
LBB7_16:
	cbz	x8, LBB7_93
	ldrb	w12, [x9], #1
	sub	w12, w12, #48
	cmp	w12, #9
	b.hi	LBB7_25
	umulh	x13, x10, x11
	cmp	xzr, x13
	cset	w13, ne
	add	x10, x10, x10, lsl #2
	lsl	x10, x10, #1
	adds	x10, x10, w12, uxtw
	cset	w12, hs
	tbnz	w13, #0, LBB7_25
	sub	x8, x8, #1
	tbz	w12, #0, LBB7_16
	b	LBB7_25
LBB7_20:
	mov	x9, x0
	mov	x8, x10
	cmp	x10, #17
	b.hs	LBB7_15
LBB7_21:
	mov	x26, #0
	mov	w10, #10
LBB7_22:
	ldrb	w11, [x9], #1
	sub	w11, w11, #48
	cmp	w11, #9
	b.hi	LBB7_24
	mul	x12, x26, x10
	add	x26, x12, w11, uxtw
	subs	x8, x8, #1
	b.ne	LBB7_22
	b	LBB7_25
LBB7_24:
	mov	w26, #4096
LBB7_25:
	cbz	x1, LBB7_27
LBB7_26:
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_27:
	str	x26, [sp, #200]
	ldr	x8, [sp, #224]
	ldr	x9, [sp, #240]
	subs	x9, x9, x8
	b.eq	LBB7_32
	mov	x10, #-6148914691236517206
	movk	x10, #43691
	umulh	x9, x9, x10
	lsr	x19, x9, #4
	add	x20, x8, #8
	b	LBB7_30
LBB7_29:
	add	x20, x20, #24
	subs	x19, x19, #1
	b.eq	LBB7_32
LBB7_30:
	ldur	x1, [x20, #-8]
	cbz	x1, LBB7_29
	ldr	x0, [x20]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB7_29
LBB7_32:
	ldr	x8, [sp, #232]
	cbz	x8, LBB7_34
	ldr	x0, [sp, #216]
	add	x8, x8, x8, lsl #1
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_34:
	add	x8, sp, #216
	bl	__ZN3std3env4args17hcf8cb98291c76d05E
Ltmp6:
	sub	x8, x29, #152
	add	x0, sp, #216
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp7:
	ldur	x1, [x29, #-152]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.eq	LBB7_44
	cbz	x1, LBB7_38
	ldur	x0, [x29, #-144]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_38:
Ltmp8:
	sub	x8, x29, #152
	add	x0, sp, #216
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp9:
	ldur	x1, [x29, #-152]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.eq	LBB7_44
	cbz	x1, LBB7_42
	ldur	x0, [x29, #-144]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_42:
Ltmp11:
	sub	x8, x29, #152
	add	x0, sp, #216
	bl	__ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h265d6abc314b1d9dE
Ltmp12:
	ldur	x1, [x29, #-152]
	mov	x8, #-9223372036854775808
	cmp	x1, x8
	b.ne	LBB7_60
LBB7_44:
	mov	w23, #20000
LBB7_45:
	str	x23, [sp, #208]
	ldr	x8, [sp, #224]
	ldr	x9, [sp, #240]
	subs	x9, x9, x8
	b.eq	LBB7_50
	mov	x10, #-6148914691236517206
	movk	x10, #43691
	umulh	x9, x9, x10
	lsr	x19, x9, #4
	add	x20, x8, #8
	b	LBB7_48
LBB7_47:
	add	x20, x20, #24
	subs	x19, x19, #1
	b.eq	LBB7_50
LBB7_48:
	ldur	x1, [x20, #-8]
	cbz	x1, LBB7_47
	ldr	x0, [x20]
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB7_47
LBB7_50:
	ldr	x8, [sp, #232]
	cbz	x8, LBB7_52
	ldr	x0, [sp, #216]
	add	x8, x8, x8, lsl #1
	lsl	x1, x8, #3
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_52:
	mov	x19, #0
	lsl	x24, x26, #3
	lsr	x8, x26, #61
	cbnz	x8, LBB7_59
	mov	x8, #9223372036854775800
	cmp	x24, x8
	b.hi	LBB7_59
	cbz	x24, LBB7_65
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	w19, #8
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_59
	mov	x25, x0
	mov	x9, x26
	stp	x23, x0, [sp, #184]
	cbz	x26, LBB7_66
LBB7_57:
	str	x9, [sp, #152]
	cmp	x26, #8
	b.hs	LBB7_67
	mov	x8, #0
	b	LBB7_70
LBB7_59:
Lloh12:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh13:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	x0, x19
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
LBB7_60:
	ldp	x0, x10, [x29, #-144]
	cbz	x10, LBB7_78
	subs	x8, x10, #1
	b.ne	LBB7_79
	ldrb	w8, [x0]
	mov	w23, #20000
	cmp	w8, #43
	b.eq	LBB7_91
	cmp	w8, #45
	b.eq	LBB7_91
	mov	w8, #1
	mov	x9, x0
	b	LBB7_87
LBB7_65:
	mov	x9, #0
	mov	w25, #8
	stp	x23, x25, [sp, #184]
	cbnz	x26, LBB7_57
LBB7_66:
	stp	xzr, xzr, [sp, #160]
	str	xzr, [sp, #176]
	mov	x8, #0
	mov	x30, #0
	mov	x24, #0
	mov	x7, #0
	mov	x6, #0
	mov	x5, #0
	mov	x3, #0
	mov	x1, #0
	mov	x16, #0
	mov	x14, #0
	mov	x12, #0
	mov	x10, #0
	mov	w0, #8
	mov	w25, #8
	mov	w23, #8
	mov	w22, #8
	mov	w21, #8
	mov	w20, #8
	mov	w19, #8
	mov	w28, #8
	mov	w27, #8
	mov	w4, #8
	mov	w2, #8
	mov	w17, #8
	mov	w15, #8
	mov	w13, #8
	mov	w11, #8
	b	LBB7_102
LBB7_67:
	and	x8, x26, #0x1ffffffffffffff8
Lloh14:
	adrp	x9, lCPI7_0@PAGE
Lloh15:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x25, #32
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
LBB7_68:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_68
	cmp	x26, x8
	b.eq	LBB7_71
LBB7_70:
	add	x9, x8, #1
	str	x9, [x25, x8, lsl #3]
	mov	x8, x9
	cmp	x26, x9
	b.ne	LBB7_70
LBB7_71:
	cbz	x24, LBB7_75
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_95
	mov	x22, x0
	mov	x21, x26
	cmp	x26, #8
	b.hs	LBB7_76
LBB7_74:
	mov	x8, #0
	b	LBB7_129
LBB7_75:
	mov	x21, #0
	mov	w22, #8
	cmp	x26, #8
	b.lo	LBB7_74
LBB7_76:
	and	x8, x26, #0x1ffffffffffffff8
Lloh16:
	adrp	x9, lCPI7_0@PAGE
Lloh17:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x22, #32
	mov	w10, #2
	dup.2d	v1, x10
	mov	w10, #4
	dup.2d	v2, x10
	mov	w10, #6
	dup.2d	v3, x10
	mov	w10, #8
	dup.2d	v4, x10
	mov	x10, x8
LBB7_77:
	add.2d	v5, v0, v1
	add.2d	v6, v0, v2
	add.2d	v7, v0, v3
	stp	q5, q6, [x9, #-32]
	add.2d	v0, v0, v4
	stp	q7, q0, [x9], #64
	subs	x10, x10, #8
	b.ne	LBB7_77
	b	LBB7_130
LBB7_78:
	mov	w23, #20000
	cbnz	x1, LBB7_92
	b	LBB7_45
LBB7_79:
	ldrb	w9, [x0]
	cmp	w9, #43
	b.ne	LBB7_86
	add	x9, x0, #1
	cmp	x10, #18
	b.lo	LBB7_87
LBB7_81:
	mov	x10, #0
	mov	w23, #20000
	mov	w11, #10
LBB7_82:
	cbz	x8, LBB7_94
	ldrb	w12, [x9], #1
	sub	w12, w12, #48
	cmp	w12, #9
	b.hi	LBB7_91
	umulh	x13, x10, x11
	cmp	xzr, x13
	cset	w13, ne
	add	x10, x10, x10, lsl #2
	lsl	x10, x10, #1
	adds	x10, x10, w12, uxtw
	cset	w12, hs
	tbnz	w13, #0, LBB7_91
	sub	x8, x8, #1
	tbz	w12, #0, LBB7_82
	b	LBB7_91
LBB7_86:
	mov	x9, x0
	mov	x8, x10
	cmp	x10, #17
	b.hs	LBB7_81
LBB7_87:
	mov	x23, #0
	mov	w10, #10
LBB7_88:
	ldrb	w11, [x9], #1
	sub	w11, w11, #48
	cmp	w11, #9
	b.hi	LBB7_90
	mul	x12, x23, x10
	add	x23, x12, w11, uxtw
	subs	x8, x8, #1
	b.ne	LBB7_88
	b	LBB7_91
LBB7_90:
	mov	w23, #20000
LBB7_91:
	cbz	x1, LBB7_45
LBB7_92:
	mov	w2, #1
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	b	LBB7_45
LBB7_93:
	mov	x26, x10
	cbnz	x1, LBB7_26
	b	LBB7_27
LBB7_94:
	mov	x23, x10
	cbnz	x1, LBB7_92
	b	LBB7_45
LBB7_95:
Ltmp14:
Lloh18:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh19:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp15:
	b	LBB7_282
LBB7_96:
Ltmp16:
	mov	x24, x0
	b	LBB7_311
LBB7_97:
Ltmp13:
	b	LBB7_101
LBB7_98:
Ltmp5:
	b	LBB7_101
LBB7_99:
Ltmp2:
	b	LBB7_101
LBB7_100:
Ltmp10:
LBB7_101:
	mov	x24, x0
	add	x0, sp, #216
	bl	__ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hd9e9aabfcd1b7715E
	mov	x0, x24
	bl	__Unwind_Resume
LBB7_102:
	str	x9, [sp, #216]
	ldr	x9, [sp, #192]
	stp	x9, x26, [sp, #224]
	stp	x10, x11, [sp, #240]
	stp	x26, x12, [sp, #256]
	stp	x13, x26, [sp, #272]
	stp	x14, x15, [sp, #288]
	stp	x26, x16, [sp, #304]
	stp	x17, x26, [sp, #320]
	stp	x1, x2, [sp, #336]
	stp	x26, x3, [sp, #352]
	stp	x4, x26, [sp, #368]
	stp	x5, x27, [sp, #384]
	stp	x26, x6, [sp, #400]
	stp	x28, x26, [sp, #416]
	stp	x7, x19, [sp, #432]
	stp	x26, x24, [sp, #448]
	stp	x20, x26, [sp, #464]
	stp	x30, x21, [sp, #480]
	stp	x26, x8, [sp, #496]
	str	x22, [sp, #512]
	str	x26, [sp, #520]
	ldp	x8, x9, [sp, #168]
	str	x9, [sp, #528]
	str	x23, [sp, #536]
	str	x26, [sp, #544]
	str	x8, [sp, #552]
	str	x25, [sp, #560]
	str	x26, [sp, #568]
	ldr	x8, [sp, #160]
	str	x8, [sp, #576]
	mov	w19, #51
	str	x0, [sp, #584]
	str	x26, [sp, #592]
	ldr	x22, [sp, #184]
LBB7_103:
	subs	w19, w19, #1
	b.eq	LBB7_105
Ltmp59:
	add	x0, sp, #216
	bl	__ZN6rust1614kernel_obvious17he2f3712ce068d2d4E
Ltmp60:
	b	LBB7_103
LBB7_105:
Ltmp62:
	bl	__ZN3std4time7Instant3now17h96378d48b1d625ebE
Ltmp63:
	stur	x0, [x29, #-232]
	stur	w1, [x29, #-224]
	add	x19, x22, #1
	add	x20, sp, #216
	sub	x21, x29, #152
LBB7_107:
	subs	x19, x19, #1
	b.eq	LBB7_109
	stur	x20, [x29, #-152]
	; InlineAsm Start
	; InlineAsm End
	ldur	x0, [x29, #-152]
Ltmp64:
	bl	__ZN6rust1614kernel_obvious17he2f3712ce068d2d4E
Ltmp65:
	b	LBB7_107
LBB7_109:
Ltmp67:
	sub	x0, x29, #232
	bl	__ZN3std4time7Instant7elapsed17h304c2773e294bb50E
Ltmp68:
	cbz	x26, LBB7_115
	ldr	x8, [sp, #232]
	ldr	x9, [sp, #304]
	sub	x10, x26, #1
	cmp	x9, x10
	csel	x10, x9, x10, lo
	cmp	x8, x10
	b.ls	LBB7_123
	cmp	x9, x10
	b.eq	LBB7_124
	ldr	x9, [sp, #296]
	ldr	x8, [sp, #224]
	cmp	x26, #8
	b.hs	LBB7_116
	mov	x19, #0
	mov	x10, #0
	b	LBB7_119
LBB7_115:
	mov	x19, #0
	b	LBB7_121
LBB7_116:
	and	x10, x26, #0x1ffffffffffffff8
	add	x11, x8, #32
	add	x12, x9, #32
	movi.2d	v0, #0000000000000000
	mov	x13, x10
	movi.2d	v1, #0000000000000000
	movi.2d	v2, #0000000000000000
	movi.2d	v3, #0000000000000000
LBB7_117:
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
	b.ne	LBB7_117
	add.2d	v0, v1, v0
	add.2d	v0, v2, v0
	add.2d	v0, v3, v0
	addp.2d	d0, v0
	fmov	x19, d0
	cmp	x26, x10
	b.eq	LBB7_121
LBB7_119:
	lsl	x11, x10, #3
	add	x9, x9, x11
	add	x8, x8, x11
	sub	x10, x26, x10
LBB7_120:
	ldr	x11, [x8], #8
	ldr	x12, [x9], #8
	eor	x11, x12, x11
	add	x19, x11, x19
	subs	x10, x10, #1
	b.ne	LBB7_120
LBB7_121:
	mov	w8, #51712
	movk	w8, #15258, lsl #16
	umulh	x9, x0, x8
	mul	x8, x0, x8
	adds	x0, x8, w1, uxtw
	cinc	x1, x9, hs
	bl	___floatuntidf
	stur	x19, [x29, #-216]
	ucvtf	d1, x26
	ucvtf	d2, x22
	fmul	d1, d1, d2
Lloh20:
	adrp	x8, __ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h55043baee9cf6639E@GOTPAGE
Lloh21:
	ldr	x8, [x8, __ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h55043baee9cf6639E@GOTPAGEOFF]
	fdiv	d0, d0, d1
	stur	d0, [x29, #-160]
	add	x9, sp, #200
	stp	x9, x8, [x29, #-152]
	add	x9, sp, #208
	stp	x9, x8, [x29, #-136]
	sub	x8, x29, #160
Lloh22:
	adrp	x9, __ZN4core3fmt5float52_$LT$impl$u20$core..fmt..Display$u20$for$u20$f64$GT$3fmt17h233df897a8cc50caE@GOTPAGE
Lloh23:
	ldr	x9, [x9, __ZN4core3fmt5float52_$LT$impl$u20$core..fmt..Display$u20$for$u20$f64$GT$3fmt17h233df897a8cc50caE@GOTPAGEOFF]
	stp	x8, x9, [x29, #-120]
Lloh24:
	adrp	x8, __ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h14d52cab6e85bc6fE@GOTPAGE
Lloh25:
	ldr	x8, [x8, __ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h14d52cab6e85bc6fE@GOTPAGEOFF]
	sub	x9, x29, #216
Lloh26:
	adrp	x10, l_anon.8883933be1ddab77f0e070c5ffe28e0c.14@PAGE
Lloh27:
	add	x10, x10, l_anon.8883933be1ddab77f0e070c5ffe28e0c.14@PAGEOFF
	stp	x9, x8, [x29, #-104]
	mov	w8, #5
	stp	x10, x8, [x29, #-208]
Lloh28:
	adrp	x8, l_anon.8883933be1ddab77f0e070c5ffe28e0c.15@PAGE
Lloh29:
	add	x8, x8, l_anon.8883933be1ddab77f0e070c5ffe28e0c.15@PAGEOFF
	mov	w9, #4
	stp	x8, x9, [x29, #-176]
	sub	x8, x29, #152
	stp	x8, x9, [x29, #-192]
Ltmp71:
	sub	x0, x29, #208
	bl	__ZN3std2io5stdio6_print17h31727a912c7756f3E
Ltmp72:
	add	x0, sp, #216
	bl	__ZN4core3ptr33drop_in_place$LT$rust16..Wide$GT$17hf9bad0f52cc3b8d9E
	add	sp, sp, #752
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
LBB7_123:
	.cfi_restore_state
Lloh30:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.16@PAGE
Lloh31:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.16@PAGEOFF
	b	LBB7_125
LBB7_124:
Lloh32:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.17@PAGE
Lloh33:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.17@PAGEOFF
	mov	x8, x9
LBB7_125:
Ltmp69:
	mov	x0, x8
	mov	x1, x8
	bl	__ZN4core9panicking18panic_bounds_check17h8935b4e0a545757eE
Ltmp70:
	b	LBB7_282
LBB7_126:
Ltmp73:
	mov	x24, x0
	add	x0, sp, #216
	bl	__ZN4core3ptr33drop_in_place$LT$rust16..Wide$GT$17hf9bad0f52cc3b8d9E
	mov	x0, x24
	bl	__Unwind_Resume
LBB7_127:
Ltmp66:
	mov	x24, x0
	add	x0, sp, #216
	bl	__ZN4core3ptr33drop_in_place$LT$rust16..Wide$GT$17hf9bad0f52cc3b8d9E
	mov	x0, x24
	bl	__Unwind_Resume
LBB7_128:
Ltmp61:
	mov	x24, x0
	add	x0, sp, #216
	bl	__ZN4core3ptr33drop_in_place$LT$rust16..Wide$GT$17hf9bad0f52cc3b8d9E
	mov	x0, x24
	bl	__Unwind_Resume
LBB7_129:
	add	x9, x8, #2
	str	x9, [x22, x8, lsl #3]
	add	x8, x8, #1
LBB7_130:
	cmp	x26, x8
	b.ne	LBB7_129
	str	x21, [sp, #144]
	str	x22, [sp, #96]
	cbz	x24, LBB7_135
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_138
	mov	x19, x0
	mov	x20, x26
	cmp	x26, #8
	b.hs	LBB7_136
LBB7_134:
	mov	x8, #0
	b	LBB7_140
LBB7_135:
	mov	x20, #0
	mov	w19, #8
	cmp	x26, #8
	b.lo	LBB7_134
LBB7_136:
	and	x8, x26, #0x1ffffffffffffff8
Lloh34:
	adrp	x9, lCPI7_0@PAGE
Lloh35:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x19, #32
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
LBB7_137:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_137
	b	LBB7_141
LBB7_138:
Ltmp17:
Lloh36:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh37:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp18:
	b	LBB7_282
LBB7_139:
Ltmp19:
	mov	x24, x0
	b	LBB7_309
LBB7_140:
	add	x9, x8, #3
	str	x9, [x19, x8, lsl #3]
	add	x8, x8, #1
LBB7_141:
	cmp	x26, x8
	b.ne	LBB7_140
	str	x20, [sp, #136]
	str	x19, [sp, #80]
	cbz	x24, LBB7_146
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_149
	mov	x28, x0
	mov	x27, x26
	cmp	x26, #8
	b.hs	LBB7_147
LBB7_145:
	mov	x8, #0
	b	LBB7_151
LBB7_146:
	mov	x27, #0
	mov	w28, #8
	cmp	x26, #8
	b.lo	LBB7_145
LBB7_147:
	and	x8, x26, #0x1ffffffffffffff8
Lloh38:
	adrp	x9, lCPI7_0@PAGE
Lloh39:
	ldr	q2, [x9, lCPI7_0@PAGEOFF]
	add	x9, x28, #32
	mov	w10, #4
	dup.2d	v0, x10
	mov	w10, #6
	dup.2d	v1, x10
	mov	w10, #8
	dup.2d	v3, x10
	mov	w10, #10
	dup.2d	v4, x10
	mov	x10, x8
LBB7_148:
	add.2d	v5, v2, v0
	add.2d	v6, v2, v1
	add.2d	v7, v2, v4
	stp	q5, q6, [x9, #-32]
	add.2d	v2, v2, v3
	stp	q2, q7, [x9], #64
	subs	x10, x10, #8
	b.ne	LBB7_148
	b	LBB7_152
LBB7_149:
Ltmp20:
Lloh40:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh41:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp21:
	b	LBB7_282
LBB7_150:
Ltmp22:
	mov	x24, x0
	b	LBB7_307
LBB7_151:
	add	x9, x8, #4
	str	x9, [x28, x8, lsl #3]
	add	x8, x8, #1
LBB7_152:
	cmp	x26, x8
	b.ne	LBB7_151
	str	x27, [sp, #128]
	str	x28, [sp, #72]
	cbz	x24, LBB7_157
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_160
	str	x26, [sp, #120]
	cmp	x26, #8
	b.hs	LBB7_158
LBB7_156:
	mov	x8, #0
	b	LBB7_162
LBB7_157:
	mov	x8, #0
	mov	w0, #8
	str	x8, [sp, #120]
	cmp	x26, #8
	b.lo	LBB7_156
LBB7_158:
	and	x8, x26, #0x1ffffffffffffff8
Lloh42:
	adrp	x9, lCPI7_0@PAGE
Lloh43:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x0, #32
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
LBB7_159:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_159
	b	LBB7_163
LBB7_160:
Ltmp23:
Lloh44:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh45:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp24:
	b	LBB7_282
LBB7_161:
Ltmp25:
	mov	x24, x0
	b	LBB7_305
LBB7_162:
	add	x9, x8, #5
	str	x9, [x0, x8, lsl #3]
	add	x8, x8, #1
LBB7_163:
	cmp	x26, x8
	b.ne	LBB7_162
	str	x0, [sp, #56]
	cbz	x24, LBB7_168
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_171
	str	x26, [sp, #112]
	cmp	x26, #8
	b.hs	LBB7_169
LBB7_167:
	mov	x8, #0
	b	LBB7_173
LBB7_168:
	mov	x8, #0
	mov	w0, #8
	str	x8, [sp, #112]
	cmp	x26, #8
	b.lo	LBB7_167
LBB7_169:
	and	x8, x26, #0x1ffffffffffffff8
Lloh46:
	adrp	x9, lCPI7_0@PAGE
Lloh47:
	ldr	q3, [x9, lCPI7_0@PAGEOFF]
	add	x9, x0, #32
	mov	w10, #6
	dup.2d	v0, x10
	mov	w10, #8
	dup.2d	v1, x10
	mov	w10, #10
	dup.2d	v2, x10
	mov	w10, #12
	dup.2d	v4, x10
	mov	x10, x8
LBB7_170:
	add.2d	v5, v3, v0
	add.2d	v6, v3, v2
	add.2d	v7, v3, v4
	add.2d	v3, v3, v1
	stp	q5, q3, [x9, #-32]
	stp	q6, q7, [x9], #64
	subs	x10, x10, #8
	b.ne	LBB7_170
	b	LBB7_174
LBB7_171:
Ltmp26:
Lloh48:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh49:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp27:
	b	LBB7_282
LBB7_172:
Ltmp28:
	mov	x24, x0
	b	LBB7_303
LBB7_173:
	add	x9, x8, #6
	str	x9, [x0, x8, lsl #3]
	add	x8, x8, #1
LBB7_174:
	cmp	x26, x8
	b.ne	LBB7_173
	str	x0, [sp, #48]
	cbz	x24, LBB7_179
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_182
	str	x26, [sp, #104]
	cmp	x26, #8
	b.hs	LBB7_180
LBB7_178:
	mov	x8, #0
	b	LBB7_184
LBB7_179:
	mov	x8, #0
	mov	w0, #8
	str	x8, [sp, #104]
	cmp	x26, #8
	b.lo	LBB7_178
LBB7_180:
	and	x8, x26, #0x1ffffffffffffff8
Lloh50:
	adrp	x9, lCPI7_0@PAGE
Lloh51:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x0, #32
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
LBB7_181:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_181
	b	LBB7_185
LBB7_182:
Ltmp29:
Lloh52:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh53:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp30:
	b	LBB7_282
LBB7_183:
Ltmp31:
	mov	x24, x0
	b	LBB7_301
LBB7_184:
	add	x9, x8, #7
	str	x9, [x0, x8, lsl #3]
	add	x8, x8, #1
LBB7_185:
	cmp	x26, x8
	b.ne	LBB7_184
	str	x0, [sp, #32]
	cbz	x24, LBB7_190
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_193
	mov	x27, x0
	str	x26, [sp, #88]
	cmp	x26, #8
	b.hs	LBB7_191
LBB7_189:
	mov	x8, #0
	b	LBB7_195
LBB7_190:
	mov	x8, #0
	mov	w27, #8
	str	x8, [sp, #88]
	cmp	x26, #8
	b.lo	LBB7_189
LBB7_191:
	and	x8, x26, #0x1ffffffffffffff8
Lloh54:
	adrp	x9, lCPI7_0@PAGE
Lloh55:
	ldr	q3, [x9, lCPI7_0@PAGEOFF]
	add	x9, x27, #32
	mov	w10, #8
	dup.2d	v0, x10
	mov	w10, #10
	dup.2d	v1, x10
	mov	w10, #12
	dup.2d	v2, x10
	mov	w10, #14
	dup.2d	v4, x10
	mov	x10, x8
LBB7_192:
	add.2d	v5, v3, v1
	add.2d	v6, v3, v2
	add.2d	v7, v3, v4
	add.2d	v3, v3, v0
	stp	q3, q5, [x9, #-32]
	stp	q6, q7, [x9], #64
	subs	x10, x10, #8
	b.ne	LBB7_192
	b	LBB7_196
LBB7_193:
Ltmp32:
Lloh56:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh57:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp33:
	b	LBB7_282
LBB7_194:
Ltmp34:
	mov	x24, x0
	b	LBB7_299
LBB7_195:
	add	x9, x8, #8
	str	x9, [x27, x8, lsl #3]
	add	x8, x8, #1
LBB7_196:
	cmp	x26, x8
	b.ne	LBB7_195
	cbz	x24, LBB7_201
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_204
	mov	x28, x0
	str	x26, [sp, #64]
	cmp	x26, #8
	b.hs	LBB7_202
LBB7_200:
	mov	x8, #0
	b	LBB7_206
LBB7_201:
	mov	x8, #0
	mov	w28, #8
	str	x8, [sp, #64]
	cmp	x26, #8
	b.lo	LBB7_200
LBB7_202:
	and	x8, x26, #0x1ffffffffffffff8
Lloh58:
	adrp	x9, lCPI7_0@PAGE
Lloh59:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x28, #32
	mov	w10, #9
	dup.2d	v1, x10
	mov	w10, #11
	dup.2d	v2, x10
	mov	w10, #13
	dup.2d	v3, x10
	mov	w10, #15
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB7_203:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_203
	b	LBB7_207
LBB7_204:
Ltmp35:
Lloh60:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh61:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp36:
	b	LBB7_282
LBB7_205:
Ltmp37:
	mov	x24, x0
	b	LBB7_297
LBB7_206:
	add	x9, x8, #9
	str	x9, [x28, x8, lsl #3]
	add	x8, x8, #1
LBB7_207:
	cmp	x26, x8
	b.ne	LBB7_206
	cbz	x24, LBB7_212
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_215
	mov	x19, x0
	str	x26, [sp, #40]
	cmp	x26, #8
	b.hs	LBB7_213
LBB7_211:
	mov	x8, #0
	b	LBB7_217
LBB7_212:
	mov	x8, #0
	mov	w19, #8
	str	x8, [sp, #40]
	cmp	x26, #8
	b.lo	LBB7_211
LBB7_213:
	and	x8, x26, #0x1ffffffffffffff8
Lloh62:
	adrp	x9, lCPI7_0@PAGE
Lloh63:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x19, #32
	mov	w10, #10
	dup.2d	v1, x10
	mov	w10, #12
	dup.2d	v2, x10
	mov	w10, #14
	dup.2d	v3, x10
	mov	w10, #16
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB7_214:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_214
	b	LBB7_218
LBB7_215:
Ltmp38:
Lloh64:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh65:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp39:
	b	LBB7_282
LBB7_216:
Ltmp40:
	mov	x24, x0
	b	LBB7_295
LBB7_217:
	add	x9, x8, #10
	str	x9, [x19, x8, lsl #3]
	add	x8, x8, #1
LBB7_218:
	cmp	x26, x8
	b.ne	LBB7_217
	cbz	x24, LBB7_223
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_226
	mov	x20, x0
	str	x26, [sp, #24]
	cmp	x26, #8
	b.hs	LBB7_224
LBB7_222:
	mov	x8, #0
	b	LBB7_228
LBB7_223:
	mov	x8, #0
	mov	w20, #8
	str	x8, [sp, #24]
	cmp	x26, #8
	b.lo	LBB7_222
LBB7_224:
	and	x8, x26, #0x1ffffffffffffff8
Lloh66:
	adrp	x9, lCPI7_0@PAGE
Lloh67:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x20, #32
	mov	w10, #11
	dup.2d	v1, x10
	mov	w10, #13
	dup.2d	v2, x10
	mov	w10, #15
	dup.2d	v3, x10
	mov	w10, #17
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB7_225:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_225
	b	LBB7_229
LBB7_226:
Ltmp41:
Lloh68:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh69:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp42:
	b	LBB7_282
LBB7_227:
Ltmp43:
	mov	x24, x0
	b	LBB7_293
LBB7_228:
	add	x9, x8, #11
	str	x9, [x20, x8, lsl #3]
	add	x8, x8, #1
LBB7_229:
	cmp	x26, x8
	b.ne	LBB7_228
	cbz	x24, LBB7_234
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_237
	mov	x21, x0
	str	x26, [sp, #16]
	cmp	x26, #8
	b.hs	LBB7_235
LBB7_233:
	mov	x8, #0
	b	LBB7_239
LBB7_234:
	mov	x8, #0
	mov	w21, #8
	str	x8, [sp, #16]
	cmp	x26, #8
	b.lo	LBB7_233
LBB7_235:
	and	x8, x26, #0x1ffffffffffffff8
Lloh70:
	adrp	x9, lCPI7_0@PAGE
Lloh71:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x21, #32
	mov	w10, #12
	dup.2d	v1, x10
	mov	w10, #14
	dup.2d	v2, x10
	mov	w10, #16
	dup.2d	v3, x10
	mov	w10, #18
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB7_236:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_236
	b	LBB7_240
LBB7_237:
Ltmp44:
Lloh72:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh73:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp45:
	b	LBB7_282
LBB7_238:
Ltmp46:
	mov	x24, x0
	b	LBB7_291
LBB7_239:
	add	x9, x8, #12
	str	x9, [x21, x8, lsl #3]
	add	x8, x8, #1
LBB7_240:
	cmp	x26, x8
	b.ne	LBB7_239
	cbz	x24, LBB7_245
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_248
	mov	x22, x0
	str	x26, [sp, #8]
	cmp	x26, #8
	b.hs	LBB7_246
LBB7_244:
	mov	x8, #0
	b	LBB7_250
LBB7_245:
	mov	x8, #0
	mov	w22, #8
	str	x8, [sp, #8]
	cmp	x26, #8
	b.lo	LBB7_244
LBB7_246:
	and	x8, x26, #0x1ffffffffffffff8
Lloh74:
	adrp	x9, lCPI7_0@PAGE
Lloh75:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x22, #32
	mov	w10, #13
	dup.2d	v1, x10
	mov	w10, #15
	dup.2d	v2, x10
	mov	w10, #17
	dup.2d	v3, x10
	mov	w10, #19
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB7_247:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_247
	b	LBB7_251
LBB7_248:
Ltmp47:
Lloh76:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh77:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp48:
	b	LBB7_282
LBB7_249:
Ltmp49:
	mov	x24, x0
	b	LBB7_289
LBB7_250:
	add	x9, x8, #13
	str	x9, [x22, x8, lsl #3]
	add	x8, x8, #1
LBB7_251:
	cmp	x26, x8
	b.ne	LBB7_250
	cbz	x24, LBB7_256
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_259
	mov	x23, x0
	str	x26, [sp, #176]
	cmp	x26, #8
	b.hs	LBB7_257
LBB7_255:
	mov	x8, #0
	b	LBB7_261
LBB7_256:
	mov	x8, #0
	mov	w23, #8
	str	x8, [sp, #176]
	cmp	x26, #8
	b.lo	LBB7_255
LBB7_257:
	and	x8, x26, #0x1ffffffffffffff8
Lloh78:
	adrp	x9, lCPI7_0@PAGE
Lloh79:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x23, #32
	mov	w10, #14
	dup.2d	v1, x10
	mov	w10, #16
	dup.2d	v2, x10
	mov	w10, #18
	dup.2d	v3, x10
	mov	w10, #20
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB7_258:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_258
	b	LBB7_262
LBB7_259:
Ltmp50:
Lloh80:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh81:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp51:
	b	LBB7_282
LBB7_260:
Ltmp52:
	mov	x24, x0
	b	LBB7_287
LBB7_261:
	add	x9, x8, #14
	str	x9, [x23, x8, lsl #3]
	add	x8, x8, #1
LBB7_262:
	cmp	x26, x8
	b.ne	LBB7_261
	cbz	x24, LBB7_267
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_270
	mov	x25, x0
	str	x26, [sp, #168]
	cmp	x26, #8
	b.hs	LBB7_268
LBB7_266:
	mov	x8, #0
	b	LBB7_272
LBB7_267:
	mov	x8, #0
	mov	w25, #8
	str	x8, [sp, #168]
	cmp	x26, #8
	b.lo	LBB7_266
LBB7_268:
	and	x8, x26, #0x1ffffffffffffff8
Lloh82:
	adrp	x9, lCPI7_0@PAGE
Lloh83:
	ldr	q0, [x9, lCPI7_0@PAGEOFF]
	add	x9, x25, #32
	mov	w10, #15
	dup.2d	v1, x10
	mov	w10, #17
	dup.2d	v2, x10
	mov	w10, #19
	dup.2d	v3, x10
	mov	w10, #21
	dup.2d	v4, x10
	mov	w10, #8
	dup.2d	v5, x10
	mov	x10, x8
LBB7_269:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x9, #-32]
	stp	q16, q17, [x9], #64
	add.2d	v0, v0, v5
	subs	x10, x10, #8
	b.ne	LBB7_269
	b	LBB7_273
LBB7_270:
Ltmp53:
Lloh84:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh85:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp54:
	b	LBB7_282
LBB7_271:
Ltmp55:
	mov	x24, x0
	b	LBB7_285
LBB7_272:
	add	x9, x8, #15
	str	x9, [x25, x8, lsl #3]
	add	x8, x8, #1
LBB7_273:
	cmp	x26, x8
	b.ne	LBB7_272
	cbz	x24, LBB7_278
	bl	__RNvCskdKJRKLKjqM_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	x0, x24
	mov	w1, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc12___rust_alloc
	cbz	x0, LBB7_281
	cmp	x26, #8
	str	x26, [sp, #160]
	b.hs	LBB7_279
LBB7_277:
	mov	x9, #0
	b	LBB7_314
LBB7_278:
	mov	x8, #0
	mov	w0, #8
	cmp	x26, #8
	str	x8, [sp, #160]
	b.lo	LBB7_277
LBB7_279:
	and	x9, x26, #0x1ffffffffffffff8
Lloh86:
	adrp	x10, lCPI7_0@PAGE
Lloh87:
	ldr	q0, [x10, lCPI7_0@PAGEOFF]
	add	x10, x0, #32
	mov	w11, #16
	dup.2d	v1, x11
	mov	w11, #18
	dup.2d	v2, x11
	mov	w11, #20
	dup.2d	v3, x11
	mov	w11, #22
	dup.2d	v4, x11
	mov	w11, #8
	dup.2d	v5, x11
	mov	x11, x9
LBB7_280:
	add.2d	v6, v0, v1
	add.2d	v7, v0, v2
	add.2d	v16, v0, v3
	add.2d	v17, v0, v4
	stp	q6, q7, [x10, #-32]
	stp	q16, q17, [x10], #64
	add.2d	v0, v0, v5
	subs	x11, x11, #8
	b.ne	LBB7_280
	b	LBB7_315
LBB7_281:
Ltmp56:
Lloh88:
	adrp	x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGE
Lloh89:
	add	x2, x2, l_anon.8883933be1ddab77f0e070c5ffe28e0c.2@PAGEOFF
	mov	w0, #8
	mov	x1, x24
	bl	__ZN5alloc7raw_vec12handle_error17hda5280d2e54fe328E
Ltmp57:
LBB7_282:
	brk	#0x1
LBB7_283:
Ltmp58:
	mov	x24, x0
	ldr	x8, [sp, #168]
	cbz	x8, LBB7_285
	ldr	x8, [sp, #168]
	lsl	x1, x8, #3
	mov	x0, x25
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_285:
	ldr	x8, [sp, #176]
	cbz	x8, LBB7_287
	ldr	x8, [sp, #176]
	lsl	x1, x8, #3
	mov	x0, x23
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_287:
	ldr	x8, [sp, #8]
	cbz	x8, LBB7_289
	ldr	x8, [sp, #8]
	lsl	x1, x8, #3
	mov	x0, x22
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_289:
	ldr	x8, [sp, #16]
	cbz	x8, LBB7_291
	ldr	x8, [sp, #16]
	lsl	x1, x8, #3
	mov	x0, x21
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_291:
	ldr	x8, [sp, #24]
	cbz	x8, LBB7_293
	ldr	x8, [sp, #24]
	lsl	x1, x8, #3
	mov	x0, x20
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_293:
	ldr	x8, [sp, #40]
	cbz	x8, LBB7_295
	ldr	x8, [sp, #40]
	lsl	x1, x8, #3
	mov	x0, x19
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_295:
	ldr	x8, [sp, #64]
	cbz	x8, LBB7_297
	ldr	x8, [sp, #64]
	lsl	x1, x8, #3
	mov	x0, x28
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_297:
	ldr	x8, [sp, #88]
	cbz	x8, LBB7_299
	ldr	x8, [sp, #88]
	lsl	x1, x8, #3
	mov	x0, x27
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_299:
	ldr	x8, [sp, #104]
	cbz	x8, LBB7_301
	ldr	x8, [sp, #104]
	lsl	x1, x8, #3
	ldr	x0, [sp, #32]
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_301:
	ldr	x8, [sp, #112]
	cbz	x8, LBB7_303
	ldr	x8, [sp, #112]
	lsl	x1, x8, #3
	ldr	x0, [sp, #48]
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_303:
	ldr	x8, [sp, #120]
	cbz	x8, LBB7_305
	ldr	x8, [sp, #120]
	lsl	x1, x8, #3
	ldr	x0, [sp, #56]
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_305:
	ldr	x8, [sp, #128]
	cbz	x8, LBB7_307
	ldr	x8, [sp, #128]
	lsl	x1, x8, #3
	ldr	x0, [sp, #72]
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_307:
	ldr	x8, [sp, #136]
	cbz	x8, LBB7_309
	ldr	x8, [sp, #136]
	lsl	x1, x8, #3
	ldr	x0, [sp, #80]
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_309:
	ldr	x8, [sp, #144]
	cbz	x8, LBB7_311
	ldr	x8, [sp, #144]
	lsl	x1, x8, #3
	ldr	x0, [sp, #96]
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
LBB7_311:
	ldr	x8, [sp, #152]
	cbnz	x8, LBB7_313
	mov	x0, x24
	bl	__Unwind_Resume
LBB7_313:
	ldr	x8, [sp, #152]
	lsl	x1, x8, #3
	ldr	x0, [sp, #192]
	mov	w2, #8
	bl	__RNvCskdKJRKLKjqM_7___rustc14___rust_dealloc
	mov	x0, x24
	bl	__Unwind_Resume
LBB7_314:
	add	x10, x9, #16
	str	x10, [x0, x9, lsl #3]
	add	x9, x9, #1
LBB7_315:
	cmp	x26, x9
	b.ne	LBB7_314
	ldp	x10, x9, [sp, #144]
	ldp	x11, x3, [sp, #96]
	ldp	x14, x12, [sp, #128]
	ldp	x15, x13, [sp, #72]
	ldp	x1, x16, [sp, #112]
	ldp	x2, x17, [sp, #48]
	ldp	x4, x7, [sp, #32]
	ldr	x5, [sp, #88]
	ldr	x6, [sp, #64]
	ldp	x30, x24, [sp, #16]
	ldr	x8, [sp, #8]
	b	LBB7_102
	.loh AdrpAdd	Lloh12, Lloh13
	.loh AdrpLdr	Lloh14, Lloh15
	.loh AdrpLdr	Lloh16, Lloh17
	.loh AdrpAdd	Lloh18, Lloh19
	.loh AdrpAdd	Lloh28, Lloh29
	.loh AdrpAdd	Lloh26, Lloh27
	.loh AdrpLdrGot	Lloh24, Lloh25
	.loh AdrpLdrGot	Lloh22, Lloh23
	.loh AdrpLdrGot	Lloh20, Lloh21
	.loh AdrpAdd	Lloh30, Lloh31
	.loh AdrpAdd	Lloh32, Lloh33
	.loh AdrpLdr	Lloh34, Lloh35
	.loh AdrpAdd	Lloh36, Lloh37
	.loh AdrpLdr	Lloh38, Lloh39
	.loh AdrpAdd	Lloh40, Lloh41
	.loh AdrpLdr	Lloh42, Lloh43
	.loh AdrpAdd	Lloh44, Lloh45
	.loh AdrpLdr	Lloh46, Lloh47
	.loh AdrpAdd	Lloh48, Lloh49
	.loh AdrpLdr	Lloh50, Lloh51
	.loh AdrpAdd	Lloh52, Lloh53
	.loh AdrpLdr	Lloh54, Lloh55
	.loh AdrpAdd	Lloh56, Lloh57
	.loh AdrpLdr	Lloh58, Lloh59
	.loh AdrpAdd	Lloh60, Lloh61
	.loh AdrpLdr	Lloh62, Lloh63
	.loh AdrpAdd	Lloh64, Lloh65
	.loh AdrpLdr	Lloh66, Lloh67
	.loh AdrpAdd	Lloh68, Lloh69
	.loh AdrpLdr	Lloh70, Lloh71
	.loh AdrpAdd	Lloh72, Lloh73
	.loh AdrpLdr	Lloh74, Lloh75
	.loh AdrpAdd	Lloh76, Lloh77
	.loh AdrpLdr	Lloh78, Lloh79
	.loh AdrpAdd	Lloh80, Lloh81
	.loh AdrpLdr	Lloh82, Lloh83
	.loh AdrpAdd	Lloh84, Lloh85
	.loh AdrpLdr	Lloh86, Lloh87
	.loh AdrpAdd	Lloh88, Lloh89
Lfunc_end0:
	.cfi_endproc
	.section	__TEXT,__gcc_except_tab
	.p2align	2, 0x0
GCC_except_table7:
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
	.uleb128 Ltmp3-Lfunc_begin0
	.uleb128 Ltmp4-Ltmp3
	.uleb128 Ltmp5-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp4-Lfunc_begin0
	.uleb128 Ltmp6-Ltmp4
	.byte	0
	.byte	0
	.uleb128 Ltmp6-Lfunc_begin0
	.uleb128 Ltmp9-Ltmp6
	.uleb128 Ltmp10-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp11-Lfunc_begin0
	.uleb128 Ltmp12-Ltmp11
	.uleb128 Ltmp13-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp12-Lfunc_begin0
	.uleb128 Ltmp14-Ltmp12
	.byte	0
	.byte	0
	.uleb128 Ltmp14-Lfunc_begin0
	.uleb128 Ltmp15-Ltmp14
	.uleb128 Ltmp16-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp15-Lfunc_begin0
	.uleb128 Ltmp59-Ltmp15
	.byte	0
	.byte	0
	.uleb128 Ltmp59-Lfunc_begin0
	.uleb128 Ltmp60-Ltmp59
	.uleb128 Ltmp61-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp62-Lfunc_begin0
	.uleb128 Ltmp63-Ltmp62
	.uleb128 Ltmp73-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp64-Lfunc_begin0
	.uleb128 Ltmp65-Ltmp64
	.uleb128 Ltmp66-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp67-Lfunc_begin0
	.uleb128 Ltmp68-Ltmp67
	.uleb128 Ltmp73-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp68-Lfunc_begin0
	.uleb128 Ltmp71-Ltmp68
	.byte	0
	.byte	0
	.uleb128 Ltmp71-Lfunc_begin0
	.uleb128 Ltmp70-Ltmp71
	.uleb128 Ltmp73-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp70-Lfunc_begin0
	.uleb128 Ltmp17-Ltmp70
	.byte	0
	.byte	0
	.uleb128 Ltmp17-Lfunc_begin0
	.uleb128 Ltmp18-Ltmp17
	.uleb128 Ltmp19-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp20-Lfunc_begin0
	.uleb128 Ltmp21-Ltmp20
	.uleb128 Ltmp22-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp23-Lfunc_begin0
	.uleb128 Ltmp24-Ltmp23
	.uleb128 Ltmp25-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp26-Lfunc_begin0
	.uleb128 Ltmp27-Ltmp26
	.uleb128 Ltmp28-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp29-Lfunc_begin0
	.uleb128 Ltmp30-Ltmp29
	.uleb128 Ltmp31-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp32-Lfunc_begin0
	.uleb128 Ltmp33-Ltmp32
	.uleb128 Ltmp34-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp35-Lfunc_begin0
	.uleb128 Ltmp36-Ltmp35
	.uleb128 Ltmp37-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp38-Lfunc_begin0
	.uleb128 Ltmp39-Ltmp38
	.uleb128 Ltmp40-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp41-Lfunc_begin0
	.uleb128 Ltmp42-Ltmp41
	.uleb128 Ltmp43-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp44-Lfunc_begin0
	.uleb128 Ltmp45-Ltmp44
	.uleb128 Ltmp46-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp47-Lfunc_begin0
	.uleb128 Ltmp48-Ltmp47
	.uleb128 Ltmp49-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp50-Lfunc_begin0
	.uleb128 Ltmp51-Ltmp50
	.uleb128 Ltmp52-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp53-Lfunc_begin0
	.uleb128 Ltmp54-Ltmp53
	.uleb128 Ltmp55-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp56-Lfunc_begin0
	.uleb128 Ltmp57-Ltmp56
	.uleb128 Ltmp58-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp57-Lfunc_begin0
	.uleb128 Lfunc_end0-Ltmp57
	.byte	0
	.byte	0
Lcst_end0:
	.p2align	2, 0x0

	.section	__TEXT,__text,regular,pure_instructions
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
Lloh90:
	adrp	x8, __ZN6rust164main17hdcfe507bfc2afffeE@PAGE
Lloh91:
	add	x8, x8, __ZN6rust164main17hdcfe507bfc2afffeE@PAGEOFF
	str	x8, [sp, #8]
Lloh92:
	adrp	x1, l_anon.8883933be1ddab77f0e070c5ffe28e0c.0@PAGE
Lloh93:
	add	x1, x1, l_anon.8883933be1ddab77f0e070c5ffe28e0c.0@PAGEOFF
	add	x0, sp, #8
	mov	w4, #0
	bl	__ZN3std2rt19lang_start_internal17hd700ba983d3377dcE
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	ret
	.loh AdrpAdd	Lloh92, Lloh93
	.loh AdrpAdd	Lloh90, Lloh91
	.cfi_endproc

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.0:
	.asciz	"\000\000\000\000\000\000\000\000\b\000\000\000\000\000\000\000\b\000\000\000\000\000\000"
	.quad	__ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h59c06508251e6ee3E
	.quad	__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h7ac6eab5420d5d32E
	.quad	__ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h7ac6eab5420d5d32E

	.section	__TEXT,__cstring,cstring_literals
l_anon.8883933be1ddab77f0e070c5ffe28e0c.1:
	.asciz	"<local-home>/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.2:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.1
	.asciz	"}\000\000\000\000\000\000\000\353\007\000\000\t\000\000"

	.section	__TEXT,__cstring,cstring_literals
l_anon.8883933be1ddab77f0e070c5ffe28e0c.3:
	.asciz	"rust16.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.4:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.3
	.asciz	"\t\000\000\000\000\000\000\000\t\000\000\000D\000\000"

	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.5:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.3
	.asciz	"\t\000\000\000\000\000\000\000\t\000\000\000[\000\000"

	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.6:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.3
	.asciz	"\t\000\000\000\000\000\000\000\n\000\000\000-\000\000"

	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.7:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.3
	.asciz	"\t\000\000\000\000\000\000\000\n\000\000\000D\000\000"

	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.8:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.3
	.asciz	"\t\000\000\000\000\000\000\000\n\000\000\000[\000\000"

	.section	__TEXT,__const
l_anon.8883933be1ddab77f0e070c5ffe28e0c.9:
	.ascii	"rust16-obvious: n="

l_anon.8883933be1ddab77f0e070c5ffe28e0c.10:
	.ascii	" k="

l_anon.8883933be1ddab77f0e070c5ffe28e0c.11:
	.ascii	" ns/elem="

l_anon.8883933be1ddab77f0e070c5ffe28e0c.12:
	.ascii	" checksum="

l_anon.8883933be1ddab77f0e070c5ffe28e0c.13:
	.byte	10

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.14:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.9
	.asciz	"\022\000\000\000\000\000\000"
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.10
	.asciz	"\003\000\000\000\000\000\000"
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.11
	.asciz	"\t\000\000\000\000\000\000"
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.12
	.asciz	"\n\000\000\000\000\000\000"
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.13
	.asciz	"\001\000\000\000\000\000\000"

	.section	__TEXT,__const
	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.15:
	.asciz	"\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000 \000\000\340\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\001\000\000\000\000\000\000\000 \000\000\340\000\000\000\000\000\000\003\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000 \000\000\360\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\002\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\003\000\000\000\000\000\000\000 \000\000\340\000\000\000"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.16:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.3
	.asciz	"\t\000\000\000\000\000\000\000\026\000\000\000'\000\000"

	.p2align	3, 0x0
l_anon.8883933be1ddab77f0e070c5ffe28e0c.17:
	.quad	l_anon.8883933be1ddab77f0e070c5ffe28e0c.3
	.asciz	"\t\000\000\000\000\000\000\000\026\000\000\0001\000\000"

.subsections_via_symbols

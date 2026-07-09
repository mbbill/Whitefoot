	.build_version macos, 26, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_mix_n                          ; -- Begin function mix_n
	.p2align	2
_mix_n:                                 ; @mix_n
; %bb.0:                                ; %entry
	cbz	x1, LBB0_4
; %bb.1:                                ; %L8.preheader
	mov	x8, x0
	mov	x0, #0                          ; =0x0
	mov	x9, #31765                      ; =0x7c15
	movk	x9, #32586, lsl #16
	movk	x9, #31161, lsl #32
	movk	x9, #40503, lsl #48
	mov	x10, #58809                     ; =0xe5b9
	movk	x10, #7396, lsl #16
	movk	x10, #18285, lsl #32
	movk	x10, #48984, lsl #48
	mov	x11, #4587                      ; =0x11eb
	movk	x11, #4913, lsl #16
	movk	x11, #18875, lsl #32
	movk	x11, #38096, lsl #48
LBB0_2:                                 ; %L8
                                        ; =>This Inner Loop Header: Depth=1
	add	x8, x8, x9
	eor	x8, x8, x8, lsr #30
	mul	x8, x8, x10
	eor	x8, x8, x8, lsr #27
	mul	x8, x8, x11
	eor	x8, x8, x8, lsr #31
	eor	x0, x8, x0
	subs	x1, x1, #1
	b.ne	LBB0_2
; %bb.3:                                ; %L5
	ret
LBB0_4:
	mov	x0, #0                          ; =0x0
	ret
                                        ; -- End function
	.globl	_main                           ; -- Begin function main
	.p2align	2
_main:                                  ; @main
; %bb.0:                                ; %entry
	mov	x8, #0                          ; =0x0
	mov	x10, #52719                     ; =0xcdef
	movk	x10, #35243, lsl #16
	movk	x10, #17767, lsl #32
	movk	x10, #291, lsl #48
	mov	w9, #49664                      ; =0xc200
	movk	w9, #3051, lsl #16
	mov	x11, #31765                     ; =0x7c15
	movk	x11, #32586, lsl #16
	movk	x11, #31161, lsl #32
	movk	x11, #40503, lsl #48
	mov	x12, #58809                     ; =0xe5b9
	movk	x12, #7396, lsl #16
	movk	x12, #18285, lsl #32
	movk	x12, #48984, lsl #48
	mov	x13, #4587                      ; =0x11eb
	movk	x13, #4913, lsl #16
	movk	x13, #18875, lsl #32
	movk	x13, #38096, lsl #48
LBB1_1:                                 ; %L8.i
                                        ; =>This Inner Loop Header: Depth=1
	add	x10, x10, x11
	eor	x10, x10, x10, lsr #30
	mul	x10, x10, x12
	eor	x10, x10, x10, lsr #27
	mul	x10, x10, x13
	eor	x10, x10, x10, lsr #31
	eor	x8, x10, x8
	subs	x9, x9, #1
	b.ne	LBB1_1
; %bb.2:                                ; %mix_n.exit
	mov	x9, #13734                      ; =0x35a6
	movk	x9, #33771, lsl #16
	movk	x9, #19999, lsl #32
	movk	x9, #26161, lsl #48
	cmp	x8, x9
	b.ne	LBB1_4
; %bb.3:                                ; %L5
	mov	w0, #0                          ; =0x0
	ret
LBB1_4:                                 ; %trap
	brk	#0x1
                                        ; -- End function
.subsections_via_symbols

	.build_version macos, 26, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_accumulate                     ; -- Begin function accumulate
	.p2align	2
_accumulate:                            ; @accumulate
; %bb.0:                                ; %entry
	cbz	x2, LBB0_4
; %bb.1:                                ; %L6.lr.ph
	ldr	x8, [x1]
	ldr	x9, [x0]
	mov	x10, #31765                     ; =0x7c15
	movk	x10, #32586, lsl #16
	movk	x10, #31161, lsl #32
	movk	x10, #40503, lsl #48
LBB0_2:                                 ; %L6
                                        ; =>This Inner Loop Header: Depth=1
	eor	x9, x8, x9
	mul	x9, x9, x10
	subs	x2, x2, #1
	b.ne	LBB0_2
; %bb.3:                                ; %L2.L3_crit_edge
	str	x9, [x0]
LBB0_4:                                 ; %L3
	ret
                                        ; -- End function
	.globl	_main                           ; -- Begin function main
	.p2align	2
_main:                                  ; @main
; %bb.0:                                ; %entry
	mov	w8, #1                          ; =0x1
	mov	w9, #51712                      ; =0xca00
	movk	w9, #15258, lsl #16
	mov	x10, #31765                     ; =0x7c15
	movk	x10, #32586, lsl #16
	movk	x10, #31161, lsl #32
	movk	x10, #40503, lsl #48
LBB1_1:                                 ; %L6.i
                                        ; =>This Inner Loop Header: Depth=1
	eor	x8, x8, #0x3
	mul	x8, x8, x10
	subs	x9, x9, #1
	b.ne	LBB1_1
; %bb.2:                                ; %accumulate.exit
	mov	x9, #64513                      ; =0xfc01
	movk	x9, #48740, lsl #16
	movk	x9, #16374, lsl #32
	movk	x9, #31981, lsl #48
	cmp	x8, x9
	b.ne	LBB1_4
; %bb.3:                                ; %L5
	mov	w0, #0                          ; =0x0
	ret
LBB1_4:                                 ; %trap
	brk	#0x1
                                        ; -- End function
.subsections_via_symbols

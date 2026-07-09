	.build_version macos, 26, 0	sdk_version 26, 5
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_mix_n                          ; -- Begin function mix_n
	.p2align	2
_mix_n:                                 ; @mix_n
	.cfi_startproc
; %bb.0:
	cbz	x1, LBB0_4
; %bb.1:
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
LBB0_2:                                 ; =>This Inner Loop Header: Depth=1
	add	x8, x8, x9
	eor	x8, x8, x8, lsr #30
	mul	x8, x8, x10
	eor	x8, x8, x8, lsr #27
	mul	x8, x8, x11
	eor	x8, x8, x8, lsr #31
	eor	x0, x8, x0
	subs	x1, x1, #1
	b.ne	LBB0_2
; %bb.3:
	ret
LBB0_4:
	mov	x0, #0                          ; =0x0
	ret
	.cfi_endproc
                                        ; -- End function
	.globl	_main                           ; -- Begin function main
	.p2align	2
_main:                                  ; @main
	.cfi_startproc
; %bb.0:
	sub	sp, sp, #48
	stp	x20, x19, [sp, #16]             ; 16-byte Folded Spill
	stp	x29, x30, [sp, #32]             ; 16-byte Folded Spill
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	mov	x19, #0                         ; =0x0
	mov	x9, #52719                      ; =0xcdef
	movk	x9, #35243, lsl #16
	movk	x9, #17767, lsl #32
	movk	x9, #291, lsl #48
	mov	w8, #49664                      ; =0xc200
	movk	w8, #3051, lsl #16
	mov	x10, #31765                     ; =0x7c15
	movk	x10, #32586, lsl #16
	movk	x10, #31161, lsl #32
	movk	x10, #40503, lsl #48
	mov	x11, #58809                     ; =0xe5b9
	movk	x11, #7396, lsl #16
	movk	x11, #18285, lsl #32
	movk	x11, #48984, lsl #48
	mov	x12, #4587                      ; =0x11eb
	movk	x12, #4913, lsl #16
	movk	x12, #18875, lsl #32
	movk	x12, #38096, lsl #48
LBB1_1:                                 ; =>This Inner Loop Header: Depth=1
	add	x9, x9, x10
	eor	x9, x9, x9, lsr #30
	mul	x9, x9, x11
	eor	x9, x9, x9, lsr #27
	mul	x9, x9, x12
	eor	x9, x9, x9, lsr #31
	eor	x19, x9, x19
	subs	x8, x8, #1
	b.ne	LBB1_1
; %bb.2:
	str	x19, [sp]
Lloh0:
	adrp	x0, l_.str@PAGE
Lloh1:
	add	x0, x0, l_.str@PAGEOFF
	bl	_printf
	and	w0, w19, #0xff
	ldp	x29, x30, [sp, #32]             ; 16-byte Folded Reload
	ldp	x20, x19, [sp, #16]             ; 16-byte Folded Reload
	add	sp, sp, #48
	ret
	.loh AdrpAdd	Lloh0, Lloh1
	.cfi_endproc
                                        ; -- End function
	.section	__TEXT,__cstring,cstring_literals
l_.str:                                 ; @.str
	.asciz	"%llu\n"

.subsections_via_symbols

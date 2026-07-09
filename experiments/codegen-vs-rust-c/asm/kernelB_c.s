	.build_version macos, 26, 0	sdk_version 26, 5
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_accumulate_naive               ; -- Begin function accumulate_naive
	.p2align	2
_accumulate_naive:                      ; @accumulate_naive
	.cfi_startproc
; %bb.0:
	cbz	x2, LBB0_3
; %bb.1:
	ldr	x8, [x0]
	mov	x9, #31765                      ; =0x7c15
	movk	x9, #32586, lsl #16
	movk	x9, #31161, lsl #32
	movk	x9, #40503, lsl #48
LBB0_2:                                 ; =>This Inner Loop Header: Depth=1
	ldr	x10, [x1]
	eor	x8, x10, x8
	mul	x8, x8, x9
	str	x8, [x0]
	subs	x2, x2, #1
	b.ne	LBB0_2
LBB0_3:
	ret
	.cfi_endproc
                                        ; -- End function
	.globl	_accumulate_restrict            ; -- Begin function accumulate_restrict
	.p2align	2
_accumulate_restrict:                   ; @accumulate_restrict
	.cfi_startproc
; %bb.0:
	cbz	x2, LBB1_4
; %bb.1:
	ldr	x8, [x1]
	ldr	x9, [x0]
	mov	x10, #31765                     ; =0x7c15
	movk	x10, #32586, lsl #16
	movk	x10, #31161, lsl #32
	movk	x10, #40503, lsl #48
LBB1_2:                                 ; =>This Inner Loop Header: Depth=1
	eor	x9, x8, x9
	mul	x9, x9, x10
	subs	x2, x2, #1
	b.ne	LBB1_2
; %bb.3:
	str	x9, [x0]
LBB1_4:
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
	mov	w19, #1                         ; =0x1
	mov	w8, #51712                      ; =0xca00
	movk	w8, #15258, lsl #16
	mov	x9, #31765                      ; =0x7c15
	movk	x9, #32586, lsl #16
	movk	x9, #31161, lsl #32
	movk	x9, #40503, lsl #48
LBB2_1:                                 ; =>This Inner Loop Header: Depth=1
	eor	x10, x19, #0x3
	mul	x19, x10, x9
	subs	x8, x8, #1
	b.ne	LBB2_1
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

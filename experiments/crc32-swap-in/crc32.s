	.build_version macos, 26, 0
	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0                          ; -- Begin function crc32_mktab
lCPI0_0:
	.long	0                               ; 0x0
	.long	1                               ; 0x1
	.long	2                               ; 0x2
	.long	3                               ; 0x3
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_crc32_mktab
	.p2align	2
_crc32_mktab:                           ; @crc32_mktab
; %bb.0:                                ; %entry
	stp	x29, x30, [sp, #-16]!           ; 16-byte Folded Spill
	mov	w0, #1                          ; =0x1
	mov	w1, #1024                       ; =0x400
	bl	_calloc
	mov	x8, #0                          ; =0x0
	mov	w9, #50201                      ; =0xc419
	movk	w9, #1901, lsl #16
	dup.4s	v0, w9
	mov	w9, #34866                      ; =0x8832
	movk	w9, #3803, lsl #16
	dup.4s	v1, w9
	mov	w9, #4196                       ; =0x1064
	movk	w9, #7607, lsl #16
	dup.4s	v2, w9
	mov	w9, #8392                       ; =0x20c8
	movk	w9, #15214, lsl #16
	dup.4s	v3, w9
	mov	w9, #16784                      ; =0x4190
	movk	w9, #30428, lsl #16
	dup.4s	v4, w9
	mov	w9, #33568                      ; =0x8320
	movk	w9, #60856, lsl #16
	dup.4s	v5, w9
Lloh0:
	adrp	x9, lCPI0_0@PAGE
Lloh1:
	ldr	q6, [x9, lCPI0_0@PAGEOFF]
	movi.4s	v7, #1
	movi.4s	v16, #2
	movi.4s	v17, #4
	movi.4s	v18, #8
	movi.4s	v19, #16
	movi.4s	v20, #32
LBB0_1:                                 ; %vector.body
                                        ; =>This Inner Loop Header: Depth=1
	and.16b	v21, v6, v7
	cmeq.4s	v21, v21, #0
	bic.16b	v21, v0, v21
	ushr.4s	v22, v6, #6
	and.16b	v23, v6, v16
	cmeq.4s	v23, v23, #0
	bcax.16b	v21, v21, v1, v23
	and.16b	v23, v6, v17
	cmeq.4s	v23, v23, #0
	bcax.16b	v21, v21, v2, v23
	and.16b	v23, v6, v18
	cmeq.4s	v23, v23, #0
	bcax.16b	v21, v21, v3, v23
	and.16b	v23, v6, v19
	cmeq.4s	v23, v23, #0
	bic.16b	v23, v4, v23
	eor3.16b	v21, v21, v23, v22
	and.16b	v22, v6, v20
	cmeq.4s	v22, v22, #0
	bcax.16b	v22, v21, v5, v22
	and.16b	v23, v21, v7
	cmeq.4s	v23, v23, #0
	ushr.4s	v22, v22, #2
	bcax.16b	v22, v22, v4, v23
	and.16b	v21, v21, v16
	cmeq.4s	v21, v21, #0
	bcax.16b	v21, v22, v5, v21
	str	q21, [x0, x8]
	add.4s	v6, v6, v17
	add	x8, x8, #16
	cmp	x8, #1024
	b.ne	LBB0_1
; %bb.2:                                ; %L17
	mov	w1, #256                        ; =0x100
	ldp	x29, x30, [sp], #16             ; 16-byte Folded Reload
	ret
	.loh AdrpLdr	Lloh0, Lloh1
                                        ; -- End function
	.globl	_crc32_upd                      ; -- Begin function crc32_upd
	.p2align	2
_crc32_upd:                             ; @crc32_upd
; %bb.0:                                ; %entry
	cbz	x4, LBB1_5
; %bb.1:                                ; %L19.preheader
	mvn	w8, w2
LBB1_2:                                 ; %L19
                                        ; =>This Inner Loop Header: Depth=1
	ldrb	w9, [x3], #1
	eor	w9, w9, w8
	and	x9, x9, #0xff
	cmp	x1, x9
	b.ls	LBB1_6
; %bb.3:                                ; %L53
                                        ;   in Loop: Header=BB1_2 Depth=1
	ldr	w9, [x0, x9, lsl #2]
	eor	w8, w9, w8, lsr #8
	subs	x4, x4, #1
	b.ne	LBB1_2
; %bb.4:                                ; %L10.loopexit
	mvn	w2, w8
LBB1_5:                                 ; %L10
	mov	x0, x2
	ret
LBB1_6:                                 ; %trap
	brk	#0x1
                                        ; -- End function
.subsections_via_symbols

	.build_version macos, 26, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_encode                         ; -- Begin function encode
	.p2align	2
_encode:                                ; @encode
; %bb.0:                                ; %entry
	cmp	x3, #3
	b.hs	LBB0_8
; %bb.1:
	mov	x10, #0                         ; =0x0
	mov	x8, #0                          ; =0x0
	mov	x9, x3
	cmp	x9, #2
	b.eq	LBB0_17
LBB0_2:                                 ; %L9
	cmp	x9, #1
	b.ne	LBB0_25
; %bb.3:                                ; %L168
	cmp	x10, x3
	b.hs	LBB0_26
; %bb.4:                                ; %L207
	cmp	x8, x1
	b.hs	LBB0_26
; %bb.5:                                ; %L223
	ldrb	w11, [x2, x10]
	ubfiz	x9, x11, #4, #2
Lloh0:
	adrp	x12, l___const_b64@PAGE
Lloh1:
	add	x12, x12, l___const_b64@PAGEOFF
	ldrb	w9, [x12, x9]
	orr	x10, x8, #0x1
	lsr	x11, x11, #2
	ldrb	w11, [x12, x11]
	strb	w11, [x0, x8]
	cmp	x10, x1
	b.hs	LBB0_26
; %bb.6:                                ; %L228
	orr	x11, x8, #0x2
	strb	w9, [x0, x10]
	cmp	x11, x1
	b.hs	LBB0_26
; %bb.7:                                ; %L232
	orr	x10, x8, #0x3
	mov	w9, #61                         ; =0x3d
	strb	w9, [x0, x11]
	b	LBB0_23
LBB0_8:                                 ; %L16.preheader
	mov	x13, #0                         ; =0x0
	add	x8, x2, #2
	mov	w10, #1                         ; =0x1
	mov	x9, x3
Lloh2:
	adrp	x11, l___const_b64@PAGE
Lloh3:
	add	x11, x11, l___const_b64@PAGEOFF
LBB0_9:                                 ; %L16
                                        ; =>This Inner Loop Header: Depth=1
	mov	x12, x13
	sub	x13, x10, #1
	cmp	x13, x3
	b.hs	LBB0_26
; %bb.10:                               ; %L27
                                        ;   in Loop: Header=BB0_9 Depth=1
	add	x14, x13, #1
	add	x13, x13, #2
	cmp	x14, x3
	ccmp	x13, x3, #2, lo
	b.hs	LBB0_26
; %bb.11:                               ; %L124
                                        ;   in Loop: Header=BB0_9 Depth=1
	cmp	x12, x1
	b.hs	LBB0_26
; %bb.12:                               ; %L140
                                        ;   in Loop: Header=BB0_9 Depth=1
	ldurb	w13, [x8, #-2]
	and	x16, x13, #0xff
	ldurb	w14, [x8, #-1]
	ldrb	w15, [x8], #3
	lsl	w17, w14, #8
	orr	w13, w17, w13, lsl #16
	and	x17, x15, #0x3f
	mov	x4, x15
	bfi	w4, w14, #8, #8
	ubfx	x13, x13, #12, #6
	ldrb	w15, [x11, x13]
	ubfx	x13, x4, #6, #6
	ldrb	w14, [x11, x13]
	ldrb	w13, [x11, x17]
	lsr	x16, x16, #2
	ldrb	w16, [x11, x16]
	strb	w16, [x0, x12]
	add	x16, x12, #1
	cmp	x16, x1
	b.hs	LBB0_26
; %bb.13:                               ; %L145
                                        ;   in Loop: Header=BB0_9 Depth=1
	add	x17, x0, x12
	strb	w15, [x17, #1]
	add	x15, x16, #1
	cmp	x15, x1
	b.hs	LBB0_26
; %bb.14:                               ; %L150
                                        ;   in Loop: Header=BB0_9 Depth=1
	strb	w14, [x17, #2]
	add	x14, x15, #1
	cmp	x14, x1
	b.hs	LBB0_26
; %bb.15:                               ; %L155
                                        ;   in Loop: Header=BB0_9 Depth=1
	add	x15, x0, x12
	sub	x9, x9, #3
	strb	w13, [x15, #3]
	add	x10, x10, #3
	add	x13, x14, #1
	cmp	x9, #2
	b.hi	LBB0_9
; %bb.16:                               ; %L9.loopexit
	add	x8, x12, #4
	sub	x10, x10, #1
	cmp	x9, #2
	b.ne	LBB0_2
LBB0_17:                                ; %L243
	cmp	x10, x3
	b.hs	LBB0_26
; %bb.18:                               ; %L243
	add	x9, x10, #1
	cmp	x9, x3
	b.hs	LBB0_26
; %bb.19:                               ; %L256
	cmp	x8, x1
	b.hs	LBB0_26
; %bb.20:                               ; %L334
	ldrb	w10, [x2, x10]
	ldrb	w9, [x2, x9]
	and	x12, x10, #0xff
	and	w11, w9, #0xf0
	orr	w10, w11, w10, lsl #8
	ubfx	x10, x10, #4, #6
Lloh4:
	adrp	x13, l___const_b64@PAGE
Lloh5:
	add	x13, x13, l___const_b64@PAGEOFF
	ubfiz	x9, x9, #2, #4
	ldrb	w10, [x13, x10]
	ldrb	w9, [x13, x9]
	orr	x11, x8, #0x1
	lsr	x12, x12, #2
	ldrb	w12, [x13, x12]
	strb	w12, [x0, x8]
	cmp	x11, x1
	b.hs	LBB0_26
; %bb.21:                               ; %L339
	orr	x12, x8, #0x2
	strb	w10, [x0, x11]
	cmp	x12, x1
	b.hs	LBB0_26
; %bb.22:                               ; %L344
	orr	x10, x8, #0x3
	strb	w9, [x0, x12]
LBB0_23:                                ; %L344
	cmp	x10, x1
	b.hs	LBB0_26
; %bb.24:                               ; %L242.sink.split
	mov	w9, #61                         ; =0x3d
	strb	w9, [x0, x10]
	add	x8, x8, #4
LBB0_25:                                ; %L242
	mov	x0, x8
	ret
LBB0_26:                                ; %trap
	brk	#0x1
	.loh AdrpAdd	Lloh0, Lloh1
	.loh AdrpAdd	Lloh2, Lloh3
	.loh AdrpAdd	Lloh4, Lloh5
                                        ; -- End function
	.section	__TEXT,__const
	.p2align	4, 0x0                          ; @__const_b64
l___const_b64:
	.ascii	"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"

.subsections_via_symbols

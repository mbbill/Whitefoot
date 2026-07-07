	.file	"twice_read.ll"
	.functype	twice_read (i32, i32) -> (i32)
	.section	.text.twice_read,"",@
	.globl	twice_read                      # -- Begin function twice_read
	.type	twice_read,@function
twice_read:                             # @twice_read
	.functype	twice_read (i32, i32) -> (i32)
	.local  	i32
# %bb.0:                                # %entry
	block   	
	local.get	0
	i32.load	0
	local.tee	0
	i32.const	1
	i32.add 
	local.tee	2
	local.get	0
	i32.lt_s
	br_if   	0                               # 0: down to label0
# %bb.1:                                # %t5
	local.get	1
	local.get	2
	i32.store	0
	local.get	0
	i32.const	1
	i32.shl 
	return
.LBB0_2:                                # %trap
	end_block                               # label0:
	unreachable
	end_function
                                        # -- End function
	.section	.custom_section.target_features,"",@
	.int8	8
	.int8	43
	.int8	11
	.ascii	"bulk-memory"
	.int8	43
	.int8	15
	.ascii	"bulk-memory-opt"
	.int8	43
	.int8	22
	.ascii	"call-indirect-overlong"
	.int8	43
	.int8	10
	.ascii	"multivalue"
	.int8	43
	.int8	15
	.ascii	"mutable-globals"
	.int8	43
	.int8	19
	.ascii	"nontrapping-fptoint"
	.int8	43
	.int8	15
	.ascii	"reference-types"
	.int8	43
	.int8	8
	.ascii	"sign-ext"
	.section	.text.twice_read,"",@

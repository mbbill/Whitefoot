.text
.p2align 2
.globl _wf__floor_run
_wf__floor_run:
    mov x9, sp
    adrp x10, _resource_stack@PAGE
    add x10, x10, _resource_stack@PAGEOFF
    add x10, x10, #4096
    mov sp, x10
    stp x29, x30, [sp, #-32]!
    stp x19, x20, [sp, #16]
    mov x19, x9
    bl _wf__main_body
    mov x9, x19
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #32
    mov sp, x9
    ret
.zerofill __DATA,__bss,_resource_stack,4096,4

.section .init, "ax"
.global _start
_start:
.cfi_startproc
.cfi_undefined ra

.option push
.option norelax // to prevent an unsupported R_RISCV_ALIGN relocation from being generated
    la gp, __global_pointer$
.option pop

    la sp, 0x7FFF0000
    lw sp, (sp)
    li a0, 0x80000000
    add sp, a0, sp
    add s0, sp, zero
    jal zero, main
.cfi_endproc
.end

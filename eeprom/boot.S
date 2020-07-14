.section .init, "ax"
.global _start
_start:
.cfi_startproc
.cfi_undefined ra

.option push
.option norelax // to prevent an unsupported R_RISCV_ALIGN relocation from being generated
    la gp, __global_pointer$
.option pop

    li sp, 0x88000000
    add s0, sp, zero
    jal zero, main
.cfi_endproc
.end

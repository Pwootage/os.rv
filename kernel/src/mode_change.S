.section .text._machine_to_supervisor
.global _mret_direct
_mret_direct:
.cfi_startproc
.cfi_undefined ra

// Assume registers are arleady correct, other than addr

la t0, dest
csrw mepc, t0
mret

dest:
ret

.cfi_endproc
OUTPUT_ARCH(riscv)
ENTRY(_start)
SECTIONS
{
  /* Read-only sections, merged into text segment: */
  . = 0x11000000;
  .init : { KEEP (*(SORT_NONE(.init))) }
  .text : { *(.text .text.*) }
  .rodata : { *(.rodata .rodata.*) }
  /* Then read/write data */
  __DATA_BEGIN__ = .;
  .data : { *(.data .data.*) }
  __SDATA_BEGIN__ = .;
  .sdata : { 
    *(.srodata .srodata.*)
    *(.sdata .sdata.*)
  }
  /* bss */
  __bss_start = .;
  .sbss : { *(.sbss .sbss.*) }
  .bss : { *(.bss .bss.* .gnu.linkonce.b.*) }
  . = ALIGN(64 / 8);
  __BSS_END__ = .;
    __global_pointer$ = MIN(__SDATA_BEGIN__ + 0x800,
		            MAX(__DATA_BEGIN__ + 0x800, __BSS_END__ - 0x800));
  _end = .; PROVIDE (end = .);
}

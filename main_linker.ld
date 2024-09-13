ENTRY(_start)

MEMORY
{
  RAM (rwx) : ORIGIN = 0x40000000, LENGTH = 128M
}

SECTIONS
{
  .text : {
    *(.text.entry)
    *(.text)
  } > RAM

  .rodata : {
    *(.rodata)
  } > RAM

  .data : {
    *(.data)
  } > RAM

  .bss : {
    *(.bss)
  } > RAM
}

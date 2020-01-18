/* MIT license

*/

ENTRY(_start);

SECTIONS
{
    __vmem_start = 0xC0000000;
    . = 0xC0010000; /* Virtual address */
    /*. = 0x80100000;  Physical address */

    __ro_start = .;
    .text :
    {
        /*KEEP(*(.text._start)) *(.text*)*/
        KEEP(*(.text._start))
        /*. = ALIGN(4096);*/
        /* Todo: put vectortable in its own section*/
        /*KEEP(*(.text._vectortable)) */
        *(.text*)
    }
    __text_end = .;

    .rodata :
    {
        *(.rodata*)
    }
    /*. = ALIGN(65536);*/ /* align in 64Kb; maybe change this to lower resolution of pages */
    __ro_end = .;

    __data_start = .;
    .data :
    {
        *(.data*)
    }
    __data_end = .;

    .bss ALIGN(8):
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(4096);
        __bss_end = .;
    }
    /*__kernel_size = __bss_end - __ro_start;*/

    /DISCARD/ : {
        *(.comment);
        *(.gnu*);
        *(.note*);
        *(.eh_frame*);
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}

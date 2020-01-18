.equ UART_DATA, 0x44E09000
.equ UART_BASE, 0x44E00000
.equ MEM_BASE, 0x80000000   // Todo: Should be read from somewhere else
.equ PAGE_BASE, 0x80004000 
.equ VMEM_BASE, 0xC0000000  // Todo: This should come from somewhere else


//.arm
.section .text._start
.global _start, vectors_start, vectors_end
.global reset_handler_addr, undef_handler_addr, swi_handler_addr, prefetch_abort_handler_addr, data_abort_handler_addr, hyp_handler_addr, irq_handler_addr, fiq_handler_addr
_start:

mov r0, #0
//mcr p15, 0, r0, c1, c0, 0   // Write SCTLR

ldr r0, =PAGE_BASE
mov r1, #4096
mov r2, #0
1:                      // This loop zeros out the pagetable
    str r2, [r0], #4
    subs r1, r1, #1
    bgt 1b

// Now we populate the pagetable
// r0 is the pointer to the base address of the pagetable
// Unsafety: Do not change r0

// !! Mapping the kernel to identity!!

ldr r0, =PAGE_BASE
add r0, r0, #(1024*2*4)     // Physical Kernel space starts at 2GB and each 1M entry is 4 byte long; Todo: Remove explicit 2GB mark
mov r1, #5
mov r2, #0x100000           // 1M increments
ldr r3, =MEM_BASE
orr r3, r3, #0x400          // AP = 01
orr r3, r3, #0x02           // Section bits and r4 = 0x8000_0402

// Now we write the sections in the pagetable
2:
    str r3, [r0], #4    // store r4 to [r1] and incr r1 by 4
    add r3, r3, r2      // incr r4 by 1M
    subs r1, r1, #1     // r1--
    bgt 2b              // loop

// !! Mapping the kernel to 0xC000_0000!!

// 0x412 = AP = 01, Domain = 0, C = 0, B = 0, and Section bits

ldr r0, =PAGE_BASE
add r0, r0, #(1024*3*4)     // Virtual Kernel space starts at 3GB and each 1MB entry is 4 byte long; Todo: Remove explicit 3GB mark
mov r1, #512                // We map the first (haha) 512 MB of RAM to low kernel memory
mov r2, #0x100000           // 1M increments
ldr r3, =MEM_BASE
orr r3, r3, #0x400          // AP = 01
orr r3, r3, #0x02           // Section bits and r4 = 0xc000_0402
mov r4, #1
lsl r4, r4, #16             // set shareable bit to 1
orr r3, r3, r4

// Now we write the sections in the pagetable
2:
    str r3, [r0], #4    // store r4 to [r1] and incr r1 by 4
    add r3, r3, r2      // incr r4 by 1M
    subs r1, r1, #1     // r1--
    bgt 2b              // loop

// 0x412 = AP = 01, Domain = 0, C = 0, B = 0, and Section bits


// !! Mapping Uart0 !!
// For Uart0, we use the identity mapping of the 0x44E0_0000 block
// Calculation: 0x4000_0000 = 1GB
//              0x0010_0000 = 1MB
// => 0x44E0_0000 = 1024MB + 78 MB

ldr r0, =PAGE_BASE
add r0, r0, #(1024*4)     // r1 = base + 4* 1102
add r0, r0, #(78*4)
ldr r1, =UART_BASE
orr r1, r1, #0x400
orr r1, r1, #0x12           // set the execute never bit
str r1, [r0]

// Set the TTBR to the base address of the pagetable
ldr r0, =PAGE_BASE
orr r0, #0x48
mcr p15, 0, r0, c8, c7, 0 // invalidate TLB, r0 is ignored
//mcr p15, 0, r0, c7, c1, 0 // invalidate Caches
mcr p15, 0, r0, c2, c0, 0 // set TTBR0
//mcr p15, 0, r0, c2, c0, 1 // set TTBR1
mov r1, #0
mcr p15, 0, r1, c2, c0, 2 // reset TTBCR

mov  r0, #0x1           // Client permission
mcr p15, 0, r0, c3, c0, 0 // Write to DACR

// Turn on the MMU
mov r0, #0
mrc p15, 0, r0, c1, c0, 0   // Read SCTLR

mov r1, #1
lsl r1, r1, #12         // set I-bit to 0
orr r1, r1, #6          // set A and C bit to 0

//bic r0, r0, r1
orr r0, r0, #1          // set M bit to 1

mcr p15, 0, r0, c1, c0, 0   // Write SCTLR
nop
nop
nop
mrc p15, 0, r2, c2, c0, 0   // Read TLBBR
mov r2, r2

// Print 
//ldr r1, =UART_DATA
//mov r2, #'!'
//str r2, [r1]


// Set SVC stack pointer
// Todo: Do this in a more sophisticated way
// Problem: We don't want to use involved pagetables here. But we only have to setup the svc stack here.
ldr r1, = VMEM_BASE
add r1, r1, #0x8000
mov sp,r1

msr cpsr, #0x92 // go to IRQ mode with irq masked
add r1, r1, #0x2000
mov sp,r1
msr cpsr, #0x97 // go to Abort mode with irq mask
add r1, r1, #0x1000
mov sp, r1
msr cpsr, #0x13 // go to SVC mode with irq enabled

bl init
b .             // loop if we return

// Figure out, whether the later can be done more efficient:

irq_handler:
//sub lr, lr, #4
//stmfd sp!, {r0,r12, lr} // put stuff on the stack
ldr r1, =UART_DATA
mov r2, #'!'
str r2, [r1]
//bl irq_rhandler
//ldmfd sp!, {r0,r12, lr} // get stuff from the stack
b .

undef_handler:
ldr r1, =UART_DATA
mov r2, #'u'
str r2, [r1]
b .

swi_handler:
ldr r1, =UART_DATA
mov r2, #'s'
str r2, [r1]
b .

prefetch_abort_handler:
ldr r1, =UART_DATA
mov r2, #'p'
str r2, [r1]
b .

data_abort_handler:
ldr r1, =UART_DATA
mov r2, #'d'
str r2, [r1]
b .

fiq_handler:
b .

// Should have its one section?
/*.section .text._vectortable*/
vectors_start:
ldr pc, reset_handler_addr
ldr pc, undef_handler_addr
ldr pc, swi_handler_addr
ldr pc, prefetch_abort_handler_addr
ldr pc, data_abort_handler_addr
ldr pc, hyp_handler_addr
ldr pc, irq_handler_addr
ldr pc, fiq_handler_addr

reset_handler_addr: .word irq_handler
undef_handler_addr: .word undef_handler
swi_handler_addr: .word swi_handler
prefetch_abort_handler_addr: .word prefetch_abort_handler
data_abort_handler_addr: .word data_abort_handler
hyp_handler_addr: .word irq_handler
irq_handler_addr: .word irq_handler
fiq_handler_addr: .word irq_handler
vectors_end:

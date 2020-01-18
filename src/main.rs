// normal no_std, no_main features:
#![no_std]
#![no_main]


#![feature(stdsimd)]
// is needed for core::arch::arm

#![feature(global_asm)]
#![feature(asm)]
// for start function

#![feature(ptr_offset_from)]
// for comparing pointers

//#![feature(const_generics)]
// is needed for virtual memory

#![feature(alloc_error_handler)]
// For allocation

#![feature(naked_functions)]

mod driver;
mod runtime_init;
mod arch;
mod kernel;
mod bsp;

use core::panic::PanicInfo;
use core::arch::arm;

use crate::arch::cpuinfo;
use crate::arch::memory;
use crate::arch::allocator;
use crate::kernel::kernel_info;
use crate::bsp::memory_map;
use crate::arch::interrupts;
use core::fmt::Write;
use core::fmt;
use crate::driver::*;

use armv7::structures::paging;

extern crate alloc;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart0 = uart::Uart::new(memory_map::UART_BASE.as_u32());
    uart0.flush_txfifo();
    writeln!(uart0,"Kernel panic: {:?}", info).ok();
    loop {
        unsafe {arm::__wfe(); };
    }
}

#[global_allocator]
static ALLOCATOR: allocator::Dummy = allocator::Dummy;

#[alloc_error_handler]
fn alloc_error_handler(_vlayout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {}")
}

pub type Result<T> = ::core::result::Result<T,::core::fmt::Error>;

pub fn initialize() -> Result<uart::Uart> {
    // Todo: Remove 5 section identity mapping of the kernel

    let mut serial = uart::Uart::new(memory_map::UART_BASE.as_u32());

    writeln!(serial,"Kernel is running.")?;
    let offset_mapping = paging::OffsetMapping::new(kernel_info::kernel_start(), memory_map::DRAM_START, memory_map::DRAM_SIZE);

    kernel_info::print_info(&mut serial)?;
    writeln!(serial,"SP is at {:#x}", cpuinfo::get_sp())?;
    let kernel_virt_addr = kernel_info::kernel_start();
    writeln!(serial, "Virtual address {:#x} => Physical address {:#x}", kernel_virt_addr, paging::get_phys_addr(kernel_virt_addr).unwrap())?;
    let addr_index = kernel_virt_addr.base_table_index();
    let mut base_table = paging::TranslationTable::get_current_ttbr0();
    writeln!(serial, "Currently used Base Table: {:#x}", base_table)?;
    writeln!(serial, "Index of kernel is: {}", addr_index)?;
    writeln!(serial, "Section entry of kernel is: {:#x}", base_table[addr_index])?;

    writeln!(serial, "Board has {} MB RAM", memory_map::DRAM_SIZE_MB)?;


    writeln!(serial, "Kernel uses {} Frames", memory::kernel_frames())?;
    unsafe {memory::PHYSICAL_MEMORY.alloc_kernel_frames().unwrap() };
    writeln!(serial, "First frame bitmap: {:#x}", unsafe { memory::PHYSICAL_MEMORY.get_entry(0) })?;

    test_alloc(&mut serial, &mut base_table, &offset_mapping)?;
    interrupts::init(&mut serial, &mut base_table, &offset_mapping)?;

    writeln!(serial, "First frame bitmap: {:#x}", unsafe { memory::PHYSICAL_MEMORY.get_entry(0) })?;
    
    cpuinfo::print_mode(&mut serial)?;
    interrupts::print_vectortable(&mut serial)?;
    cpuinfo::print_status(&mut serial)?;

    serial.write_str("Disabling Watchdog..\n")?;
    let watchdog = watchdog::Watchdog::new(0x44e3_5000);
    watchdog.disable();
    serial.write_str("Disabled Watchdog\n")?;
    serial.write_str("Enabling Timer\n")?;
    let timer = timer::Timer::new(0x44E0_5000);
    timer.init(0x0000_0fff);
    Ok(serial)
}

pub fn test_alloc<T: fmt::Write>(serial: &mut T, mut base_table: &mut paging::TranslationTable, offset_mapping: &paging::OffsetMapping) -> fmt::Result {
    let physical_addr = unsafe { memory::PHYSICAL_MEMORY.allocate_frame().unwrap() };
    writeln!(serial, "Allocated physical address: {:#x}", physical_addr)?;
    // Get virtual address from physical
    let virtual_addr = offset_mapping.convert_phys_addr(physical_addr).unwrap();
    writeln!(serial, "Allocated virtual address: {:#x}", virtual_addr)?;
    let mut page_table = unsafe { paging::PageTable::create(virtual_addr, 1024, &mut base_table).unwrap() };
    page_table[0] = paging::PageDescriptor::new_smallpage(memory_map::UART_BASE, 0b001, 0, true, false, false, false, false).unwrap();
    let mut new_uart0 = uart::Uart::new(0x4000_0000);
    writeln!(new_uart0,"Paging is running.")?;
    Ok(())
}

pub extern fn kernel_main() -> ! {
    let mut serial = initialize().unwrap();
    serial.flush_txfifo();

    writeln!(serial,"Kernel is running.").unwrap();
    loop {
        let c = serial.getc();
        if c == 'q' {
            unsafe { asm!("bkpt") };
        }
        if c == 'p' {
            panic!("Panic!");
        }
        serial.putc(c);
    }
}

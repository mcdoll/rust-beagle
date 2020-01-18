//! The interrupt handlers

use armv7::structures::paging;
use armv7::structures::interrupts;
use armv7::regs::vmem_control::*;
use armv7::regs::security::*;
use core::fmt;
use crate::driver::uart;
use crate::kernel::kernel_info;
use crate::bsp::memory_map;
use crate::arch::cpuinfo;

#[no_mangle]
pub unsafe extern "C" fn irq_rhandler() -> () {
}

#[no_mangle]
#[naked]
pub extern "C" fn exception_handler() -> ! {
    use core::fmt::Write;
    let mut uart0 = uart::Uart::new(memory_map::UART_BASE.as_u32());
    //uart0.flush_txfifo();
    writeln!(uart0, "CPU exception").unwrap();
    cpuinfo::print_mode(&mut uart0).ok();
    loop { }
}

pub fn init<T: fmt::Write>(serial: &mut T, base_table: &mut paging::TranslationTable, _kernel_offset_mapping: &paging::OffsetMapping) -> fmt::Result {
    use armv7::*;

    writeln!(serial, "\nInitializing Interrupts.\n")?;

    // Initialize mapping of pagetable: 0x8000_5000 to 0xfffx_xxxx
    let page_table_addr = kernel_info::kernel_start() + 0x5400 as u32;
    let page_table_addr2 = kernel_info::kernel_start() + 0x5800 as u32;
    let vector_table_addr = memory_map::DRAM_START + 0x5000 as u32;
    // Add a pagetable to the 0xfffx_xxxx at virtual address 0xC000_5400
    let mut page_table_fff = unsafe { paging::PageTable::create(page_table_addr, 4095, base_table).unwrap() };
    let mut page_table_000 = unsafe { paging::PageTable::create(page_table_addr2, 0, base_table).unwrap() };
    page_table_fff[240] = paging::PageDescriptor::new_smallpage(vector_table_addr, 0b001, 0, false, false, false, false, false).unwrap();
    page_table_000[0] = paging::PageDescriptor::new_smallpage(vector_table_addr, 0b001, 0, false, false, false, false, false).unwrap();
    writeln!(serial,"Added interrupt page for interrupt table")?;

    // Define the interrupt handler
    //let handler = interrupts::Handler::default();
    //writeln!(serial, "Default interrupt handler: {:?}", handler)?;
    let vector_table = interrupts::VectorTable::new(true);
    let exc_handler = PhysicalAddress::from_ext_fn(exception_handler);
    //let irq_addr = unsafe { paging::VirtualAddress::from_ptr(&irq_handler as *const u32) };
    //writeln!(serial, "Pointer to memory: {:#x}", irq_addr)?;
    vector_table.init(exc_handler);
    //vector_table.init(irq_addr);
    //unsafe { interrupts::init_vectortable(&mut vectors_start, &mut vectors_end, 0xffff_0000 as *mut u32); }
    SCTLR.modify(SCTLR::EXCENDIAN::LittleEndian + SCTLR::THUMBEXC::Arm + SCTLR::VECENABLE::UseVectorTable + SCTLR::INSTR::Disabled + SCTLR::CACHE::Disabled);
    VBAR.set(0x8000_5000);

    //let exc_handler = unsafe { &except_handler as *const u32 as usize as u32 };
    //writeln!(serial, "Exception handler at {:#x}", kernel_offset_mapping.convert_phys_addr(exc_handler).unwrap())?;
    //let mut interrupt_table = interrupts::InterruptTable::default();
    //let mut interrupt_table = interrupts::InterruptTable::new(true);
    //let generic_exception_handler = unsafe { interrupts::Handler::new(__EXCEPTION_HANDLER) };
    //interrupt_table.set_handler(interrupts::HandlerType::Undef, generic_exception_handler);
    //interrupt_table.set_handler(interrupts::HandlerType::PrefetchAbort, generic_exception_handler);
    //interrupt_table.set_handler(interrupts::HandlerType::DataAbort, generic_exception_handler);
    //writeln!(serial, "Interrupt table: {:?}", interrupt_table)?;
    //unsafe { interrupt_table.start_table(true) };

    //writeln!(serial, "PC is at {:#x}", PC.get())?; // this would immediately call the exception function

    Ok(())
}

pub fn print_vectortable<T: fmt::Write>(serial: &mut T) -> fmt::Result {
    let table_addr = {
        if SCTLR.is_set(SCTLR::VECTOR) {
            0xffff_ff00
        } else {
            VBAR.get()
        }
    };
    writeln!(serial, "Interrupt table is at {:#x}", table_addr)?;

    // figure out whether we use hivecs
    // if not get hold of VBAR
    let table_raw = table_addr as *mut [u32; 16];
    let table = unsafe { *table_raw };
    writeln!(serial, "Interrupt table as u32:\n{:#x?}", table)?;
    Ok(())
}

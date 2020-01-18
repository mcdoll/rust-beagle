//! Infos provided by the linker script
// Author: Moritz Doll
// License: GPLv3

use armv7::VirtualAddress;
use core::fmt;

// Useful functions
pub fn print_info<T: fmt::Write>(serial: &mut T) -> fmt::Result {
    writeln!(serial,"Total allocated size for the kernel is {:#x} Bytes", kernel_size())?;
    writeln!(serial,"Loaded Kernel has size {:#x} Bytes", kernel_size_loaded())?;
    writeln!(serial,"Text section has size {:#x} Bytes", text_size())?;
    writeln!(serial,"Data section has size {:#x} Bytes", data_size())?;
    writeln!(serial,"Bss section has size {:#x} Bytes", bss_size())?;
    Ok(())
}

// Calculate the size of the kernel

unsafe fn diff(end: *const u8, start: *const u8) -> isize {
    end.offset_from(start)
}

pub fn kernel_start() -> VirtualAddress {
    extern "C" {
        static __vmem_start: u8;
    }
    unsafe { VirtualAddress::from_ptr(&__vmem_start) }
}

pub fn kernel_memory_size() -> isize {
    extern "C" {
        static __vmem_start: u8;
        static __bss_end: u8;
    }
    unsafe { diff(&__bss_end, &__vmem_start) }
}

pub fn kernel_size() -> isize {
    extern "C" {
        // size of the kernel given by linker script
        static __ro_start: u8;
        static __bss_end: u8;
    }
    unsafe { diff(&__bss_end, &__ro_start) }
}

pub fn kernel_size_loaded() -> isize {
    extern "C" {
        // size of the kernel given by linker script
        #[no_mangle]
        static __ro_start: u8;
        static __data_end: u8;
    }
    unsafe { diff(&__data_end, &__ro_start) }
}

pub fn text_size() -> isize {
    extern "C" {
        static __ro_start: u8;
        static __ro_end: u8;
    }
    unsafe { diff(&__ro_end, &__ro_start) }
}

pub fn data_size() -> isize {
    extern "C" {
        static __data_start: u8;
        static __data_end: u8;
    }
    unsafe { diff(&__data_end, &__data_start) }
}

pub fn bss_size() -> isize {
    extern "C" {
        static __bss_start: u8;
        static __bss_end: u8;
    }
    unsafe { diff(&__bss_end, &__bss_start) }
}

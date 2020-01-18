//! Memory map
// Author: Moritz Doll
// License: GPLv3


// For the Sitara:
pub use sitara::memory_map::*;

// For bcm2835:
// const DRAM_START: u32 = 0x0000_0000;

pub const DRAM_SIZE: u32 = (DRAM_END.as_u32() - DRAM_START.as_u32() + 1);
pub const DRAM_SIZE_KB: u32 = DRAM_SIZE / 1024 ;
pub const DRAM_SIZE_MB: u32 = DRAM_SIZE_KB / 1024 ;


// For bcm2835:
// const UART_BASE: u32 = 0x2020_0000;
// but we also need gpio-pins to setup uart0 on raspi-zero
//


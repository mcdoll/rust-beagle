// Author: Moritz Doll
// License: GPLv3

use armv7::regs::vmem_control::*;
use armv7::regs::core_regs::*;
use armv7::regs::program_state;
use core::fmt;

pub fn get_mmu_status() -> bool {
    //let mmu_status = SCTLR.read_as_enum(SCTLR::MMU)?;
    /*match mmu_status {
        SCTLR::MMU::Value::Enabled => Some(true),
        SCTLR::MMU::Value::Disabled => Some(false),
    }*/
    SCTLR.is_set(SCTLR::MMU)
}

pub fn get_hivecs() -> bool {
    SCTLR.is_set(SCTLR::VECTOR)
}


pub fn get_sp() -> u32 {
    SP.get()
}

pub fn print_status<T: fmt::Write>(serial: &mut T) -> fmt::Result {
    writeln!(serial, "The status of the device:")?;
    if SCTLR.is_set(SCTLR::MMU) {
        writeln!(serial, "MMU is on.")?;
    } else {
        writeln!(serial, "MMU is off.")?;
    }
    if SCTLR.is_set(SCTLR::VECTOR) {
        writeln!(serial, "Hivecs are on.")?;
    } else {
        writeln!(serial, "Hivecs are off.")?;
    }
    if SCTLR.is_set(SCTLR::EXCENDIAN) {
        write!(serial, "Exceptions are taken in big endian ")?;
    } else {
        write!(serial, "Exceptions are taken in little endian ")?;
    }
    if SCTLR.is_set(SCTLR::EXCENDIAN) {
        writeln!(serial, "and in thumb mode.")?;
    } else {
        writeln!(serial, "and in arm mode.")?;
    }
    if SCTLR.is_set(SCTLR::TEXREMAP) {
        writeln!(serial, "Tex Remap enabled.")?;
    } else {
        writeln!(serial, "Tex Remap disabled.")?;
    }
    Ok(())
}

#[inline]
pub fn print_mode<T: fmt::Write>(serial: &mut T) -> fmt::Result {
    match program_state::get_current_mode() {
        Some(mode) => writeln!(serial, "Mode is {}", mode)?,
        None => writeln!(serial, "Could not determine operating mode.")?,
    };
    Ok(())
}

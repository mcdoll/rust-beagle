#[no_mangle]
pub unsafe extern "C" fn init() -> ! {
    extern "C" {
        // Boundaries of the .bss section, provided by the linker script
        static mut __bss_start: u32;
        static mut __bss_end: u32;
    }

    // Zero out the .bss section
    r0::zero_bss(&mut __bss_start, &mut __bss_end);

    crate::kernel_main()
}
global_asm!(include_str!("init.s"));

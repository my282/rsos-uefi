#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
extern "sysv64" fn _start() -> ! {
    loop {
        hlt()
    }
}

fn hlt() {
    unsafe { asm!("hlt") }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

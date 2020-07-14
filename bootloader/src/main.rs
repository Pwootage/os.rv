#![no_std]
#![no_main]

use riscv_rt::entry;
use core::panic::PanicInfo;

#[entry]
fn main() -> ! {
    // do something here
    loop { }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // atomic::compiler_fence(Ordering::SeqCst);
    }
}

pub mod peripheral_fs;
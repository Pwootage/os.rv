#![no_std]
#![no_main]

use riscv_rt::entry;
use core::panic::PanicInfo;
use peripherals::basic_fifo::BasicFIFO;

#[entry]
fn main() -> ! {
    // do something here
    let mut fifo = BasicFIFO::panic_fifo();
    fifo.write("Hello, kernel panic!");
    fifo.write_ready();

    loop { }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // atomic::compiler_fence(Ordering::SeqCst);
    }
}

pub mod peripherals;
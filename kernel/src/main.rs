#![no_std]
#![no_main]

use riscv_rt::entry;
use core::panic::PanicInfo;
use peripherals::basic_fifo::BasicFIFO;
use core::fmt::Write;

//#[macro_use]
//extern crate alloc;

#[entry]
fn main() -> ! {
    // do something here
    panic!("Kernel ended execution")
}

pub struct PanicOut {
    pub fifo: BasicFIFO
}


#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut fifo = BasicFIFO::panic_fifo();
    // Don't care if this fails, we're literally in panic()
    let _res = write!(fifo, "{}", info);
    fifo.write_ready();
    loop {
        // atomic::compiler_fence(Ordering::SeqCst);
    }
}

pub mod peripherals;
#![no_std]
#![no_main]
// TODO: remove these supressions (it's just to hide non-useful ones early in dev)
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
// #![feature(const_generics)]

use riscv_rt::entry;
use core::panic::PanicInfo;
use peripherals::basic_fifo::BasicFIFO;
use core::fmt::Write;

//#[macro_use]
//extern crate alloc;

#[macro_use]
extern crate bitfield;

#[entry]
fn main() -> ! {
    // do something here
    mmu::setup_mmu();
    panic!("Kernel ended execution!")
}

pub struct PanicOut {
    pub fifo: BasicFIFO
}


#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut fifo = BasicFIFO::panic_fifo();
    // Don't care if this fails, we're literally in panic()
    let _ = writeln!(fifo, "{}", info);

    // let mut fp: *mut u32;
    // let mut ra: *mut u32;
    // let mut sp: *mut u32;
    // unsafe {
    //     asm!("mv {}, fp", out(reg) fp);
    //     asm!("mv {}, ra", out(reg) ra);
    //     asm!("mv {}, sp", out(reg) sp);
    //     // asm!("mv $0, a0"   : "=r"(fp));
    // }
    // let _ = writeln!(fifo, "FP: {:x}, RA: {:x}, SP: {:x}", fp as usize, ra as usize, sp as usize);
    // unsafe {
    //     let rows = 16;
    //     let cols = 16;
    //     for row in (0..rows) {
    //         sp = 0x80007000 as *mut u32;
    //         let _ = write!(fifo, "\n{:08x}: ", sp.offset(-row * rows) as usize);
    //         for col in (0..cols) {
    //             let _ = write!(fifo, "{:08x} ", *sp.offset(-row * rows + col));
    //         }
    //     }
    // }

    fifo.write_ready();
    loop {
        unsafe { riscv::asm::ebreak(); }
    }
}

pub mod peripherals;
pub mod drivers;
pub mod mmu;
use volatile_register::{RW, RO};
use core::fmt;
use crate::peripherals::stream::{InStream, OutStream};

pub struct BasicFIFO {
    p: &'static mut BasicFIFORegisters
}

#[repr(C)]
struct BasicFIFORegisters {
    pub fifo: RW<u8>,
    pub read_ready: RO<u8>,
    pub write_ready: RW<u8>
}

impl BasicFIFO {
    pub fn component_fifo() -> BasicFIFO {
        return BasicFIFO::new(0x1000_1000)
    }
    pub fn panic_fifo() -> BasicFIFO {
        return BasicFIFO::new(0x1000_2000)
    }
    pub fn print_fifo() -> BasicFIFO {
        return BasicFIFO::new(0x1000_0000)
    }

    fn new(addr: u32) -> BasicFIFO {
        BasicFIFO {
            p: unsafe { &mut *(addr as *mut BasicFIFORegisters) }
        }
    }

    pub fn write_ready(&mut self) {
        unsafe {
            self.p.write_ready.write(1);
            riscv::asm::ebreak();
        }
    }
}

impl InStream for BasicFIFO {
    fn read(&mut self) -> u8 {
        self.p.fifo.read()
    }
}

impl OutStream for BasicFIFO {
    fn write(&mut self, v: u8) {
        unsafe { self.p.fifo.write(v) }
    }

}

impl fmt::Write for BasicFIFO {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            self.write(c)
        }
        Ok(())
    }
}
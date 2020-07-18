use volatile_register::{RW, RO};
use core::fmt;

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

    fn new(addr: u32) -> BasicFIFO {
        BasicFIFO {
            p: unsafe { &mut *(addr as *mut BasicFIFORegisters) }
        }
    }

    pub fn write_ready(&mut self) {
        unsafe { 
            self.p.write_ready.write(1)
        }
    }
}

impl fmt::Write for BasicFIFO {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            unsafe { self.p.fifo.write(c) }
        }
        Ok(())
    }
}
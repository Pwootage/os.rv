use volatile_register::RO;

pub struct MemorySize {
    p: &'static mut MemorySizeRegisters
}

#[repr(C)]
struct MemorySizeRegisters {
    pub size: RO<usize>
}

impl MemorySize {
    pub fn new() -> MemorySize {
        MemorySize {
            p: unsafe { &mut *(0x7FFF_0000 as *mut MemorySizeRegisters) }
        }
    }

    pub fn max_size(&self) -> usize {
        self.p.size.read()
    }
}
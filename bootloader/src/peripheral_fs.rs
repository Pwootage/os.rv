use volatile_register::{RW, RO};

pub struct FS {
    p: &'static mut FSRegisters
}

#[repr(C)]
struct FSRegisters {
    pub id: RW<u64>
}

impl FS {
    pub fn get_id(&self) -> u64 {
        self.p.id.read()
    }

    pub fn set_id(&mut self, id: u64) {
        unsafe { self.p.id.write(id) }
    }
}

struct Peripherals {
    fs: Option<FS>,
}
impl Peripherals {
    fn take_fs(&mut self) -> FS {
        let p = replace(&mut self.fs, None);
        p.unwrap()
    }
}
static mut PERIPHERALS: Peripherals = Peripherals {
    fs: Some(FS {
        p: unsafe { &mut *(0xE000_E010 as *mut FSRegisters) }
    }),
};
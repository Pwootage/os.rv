use core::cmp::min;

pub trait OutStream {
    fn write(&mut self, v: u8);

    fn write8(&mut self, v: u8) {
        self.write(v)
    }
    fn write16(&mut self, v: u16) {
        self.write((v >> 0) as u8);
        self.write((v >> 8) as u8);
    }
    fn write32(&mut self, v: u32) {
        self.write((v >> 0) as u8);
        self.write((v >> 8) as u8);
        self.write((v >> 16) as u8);
        self.write((v >> 24) as u8);
    }
    fn write64(&mut self, v: u64) {
        self.write((v >> 0) as u8);
        self.write((v >> 8) as u8);
        self.write((v >> 16) as u8);
        self.write((v >> 24) as u8);
        self.write((v >> 32) as u8);
        self.write((v >> 40) as u8);
        self.write((v >> 48) as u8);
        self.write((v >> 56) as u8);
    }
    fn write128(&mut self, v: u128) {
        self.write((v >> 0) as u8);
        self.write((v >> 8) as u8);
        self.write((v >> 16) as u8);
        self.write((v >> 24) as u8);
        self.write((v >> 32) as u8);
        self.write((v >> 40) as u8);
        self.write((v >> 48) as u8);
        self.write((v >> 56) as u8);
        self.write((v >> 64) as u8);
        self.write((v >> 72) as u8);
        self.write((v >> 80) as u8);
        self.write((v >> 88) as u8);
        self.write((v >> 96) as u8);
        self.write((v >> 104) as u8);
        self.write((v >> 112) as u8);
        self.write((v >> 120) as u8);
    }
    fn write_bytes(&mut self, arr: &[u8]) {
        for i in 0..arr.len() {
            self.write(arr[i]);
        }
    }
}

pub trait InStream {
    fn read(&mut self) -> u8;

    fn read8(&mut self) -> u8 {
        return self.read()
    }
    fn read16(&mut self) -> u16 {
        return (self.read() as u16) |
            ((self.read() as u16) << 8)
    }
    fn read32(&mut self) -> u32 {
        return (self.read() as u32) |
            ((self.read() as u32) << 8) |
            ((self.read() as u32) << 16) |
            ((self.read() as u32) << 24)
    }
    fn read64(&mut self) -> u64 {
        return (self.read() as u64) |
            ((self.read() as u64) << 8) |
            ((self.read() as u64) << 16) |
            ((self.read() as u64) << 24) |
            ((self.read() as u64) << 32) |
            ((self.read() as u64) << 40) |
            ((self.read() as u64) << 48) |
            ((self.read() as u64) << 56)
    }
    fn read128(&mut self) -> u128 {
        return (self.read() as u128) |
            ((self.read() as u128) << 8) |
            ((self.read() as u128) << 16) |
            ((self.read() as u128) << 24) |
            ((self.read() as u128) << 32) |
            ((self.read() as u128) << 40) |
            ((self.read() as u128) << 48) |
            ((self.read() as u128) << 56) |
            ((self.read() as u128) << 64) |
            ((self.read() as u128) << 72) |
            ((self.read() as u128) << 80) |
            ((self.read() as u128) << 88) |
            ((self.read() as u128) << 96) |
            ((self.read() as u128) << 104) |
            ((self.read() as u128) << 112) |
            ((self.read() as u128) << 120)
    }

    fn read_bytes(&mut self, len: usize, arr: &mut [u8]) {
        let count = min(len, arr.len());
        for i in 0..count {
            arr[i] = self.read();
        }
        // Always read len; ignore extras
        for i in count..len {
            let _ = self.read();
        }
    }
}
use crate::peripherals::stream::{InStream, OutStream};


#[derive(Copy, Clone)]
enum TaggedBinary {
    NULL,
    Int8(u8),
    Int16(u16),
    Int32(u32),
    Int64(u64),
    Int128(u128),
    BytesStatic([u8; 256]),
    //TODO: use vec<>! Need kmalloc first
    Value(u32),
    END,
}

impl TaggedBinary {
    fn read_from(input: &mut dyn InStream) -> TaggedBinary {
        let t = input.read();
        return match t {
            0x00 => { TaggedBinary::NULL }
            0x01 => { TaggedBinary::Int8(input.read8()) }
            0x02 => { TaggedBinary::Int16(input.read16()) }
            0x03 => { TaggedBinary::Int32(input.read32()) }
            0x04 => { TaggedBinary::Int64(input.read64()) }
            0x05 => { TaggedBinary::Int128(input.read128()) }
            0x06 => {
                let len = input.read32() as usize;
                let mut arr: [u8; 256] = [0; 256];
                input.read_bytes(len, &mut arr);
                TaggedBinary::BytesStatic(arr)
            }
            0x08 => { TaggedBinary::Value(input.read32()) }
            0xFF => { TaggedBinary::END }
            _ => panic!("Unknown tagged binary type {}", t)
        };
    }

    fn write_to(&self, output: &mut dyn OutStream) {
        match self {
            TaggedBinary::NULL => {
                output.write8(0x00);
            },
            TaggedBinary::Int8(v) => {
                output.write8(0x01);
                output.write8(*v);
            },
            TaggedBinary::Int16(v) => {
                output.write8(0x02);
                output.write16(*v);
            },
            TaggedBinary::Int32(v) => {
                output.write8(0x03);
                output.write32(*v);
            },
            TaggedBinary::Int64(v) => {
                output.write8(0x04);
                output.write64(*v);
            },
            TaggedBinary::Int128(v) => {
                output.write8(0x05);
                output.write128(*v);
            },
            TaggedBinary::BytesStatic(v) => {
                output.write8(0x06);
                output.write32(v.len() as u32);
                output.write_bytes(v);
            },
            TaggedBinary::Value(v) => {
                output.write8(0x08);
                output.write32(*v);
            },
            TaggedBinary::END => {
                output.write8(0xFF);
            },
        }
    }
}
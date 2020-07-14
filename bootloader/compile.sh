#!/bin/bash -xe

TARGET="riscv64-unknown-elf"
ARCH="rv64gc"

rm -rf build
mkdir -p build

LD="${TARGET}-ld"
CC="${TARGET}-gcc"
CXX="${TARGET}-g++"
OBJDUMP="${TARGET}-objdump"
OBJCOPY="${TARGET}-objcopy"

cargo build --release

${OBJCOPY} \
  -O binary \
  target/riscv32imac-unknown-none-elf/release/hello-os-rust \
  target/riscv32imac-unknown-none-elf/release/hello-os-rust.bin



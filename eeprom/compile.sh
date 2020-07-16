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

${CXX} \
  -ffreestanding -nostartfiles -nostdlib -nodefaultlibs \
  -Wl,--gc-sections \
  -mcmodel=medany \
  -mabi=ilp32 \
  -march=rv32im \
  -T riscv64-virt.ld \
  -g \
  -O0 \
  -o build/eeprom \
  main.cpp \
  boot.S

${OBJCOPY} \
  -O binary \
  build/eeprom \
  build/eeprom.bin



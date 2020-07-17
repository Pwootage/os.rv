#!/bin/bash -xe
cargo build
cargo build --release
ls -l target/riscv32imac-unknown-none-elf/debug/kernel
ls -l target/riscv32imac-unknown-none-elf/release/kernel


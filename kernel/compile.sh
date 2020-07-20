#!/bin/bash -xe
cargo +stable build
cargo +stable build --release
ls -l target/riscv32imac-unknown-none-elf/debug/kernel
ls -l target/riscv32imac-unknown-none-elf/release/kernel


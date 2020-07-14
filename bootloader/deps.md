brew tap riscv/riscv
brew install riscv-tools
rustup target add riscv64gc-unknown-none-elf
rustup target add riscv32gc-unknown-none-elf
rustup target add riscv32imac-unknown-none-elf
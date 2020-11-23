## OS in Rust

This is a OS made in Rust following tutorials from the internet like phil-opp

First off, I struggled so much with compiling this on MacOS, because nightly sometimes
does weird stuff on new versions in OSs that are not Linux.

## Instructions Install [MacOS]

1. First you have to install a nightly version that works well when using LLVM tools. This is the version that I found working well and compatible with the requirements of this project

- `rustup toolchain install nightly-2020-10-05 && rustup default nightly-2020-10-05`

2. Install Rust tools for building in bare metal.

- `rustup component add rust-src`
- `rustup component add llvm-tools-preview`

## Instructions for building the kernel into a bootable disk image

- `cargo install bootimage`
- `cargo bootimage`

## And run on QEMU

- `qemu-system-x86_64 -drive format=raw,file=target/os/debug/bootimage-rust-os.bin`

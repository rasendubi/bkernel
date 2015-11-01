[![Build Status](https://travis-ci.org/rasendubi/bkernel.svg)](https://travis-ci.org/rasendubi/bkernel)

bkernel is an experimental kernel for embedded devices written in Rust. I'm mostly trying out Rust now to see how it applies to kernel development.

# Prerequisites

## gcc-arm-none-eabi toolchain

You need an gcc-arm-none-eabi toolchain before you can build the kernel.

If you don't know where to get one, you can get it [there](https://launchpad.net/gcc-arm-embedded/+download):

- Download one for your platform
- Unpack
- Add `<path_to_toolchain>/bin` to your `$PATH` variable

## Rust sources

bkernel needs Rust sources to build libcore for the target. If you don't have one, don't worry: it will be automatically downloaded to `rust` directory.

If you have Rust git repo on you computer, you can point to it with:

```sh
export RUSTDIR=/path/to/rust
```

Note: building bkernel will checkout rust to a commit your rustc was compiled with.

# Build instructions

Just invoke `make`.

## Make targets

- `make` build all binaries, documentation and run tests;
- `make test` run tests;
- `make doc` build documentation;
- `make kernel_meta` only build kernel.

# Flashing

If you have a STM32F4Discovery board, you can flash it in the following way:

- Run openocd with `openocd -f openocd.cfg`.
- Connect to the running server with `telnet localhost 4444` and run `flash_bkernel kernel.bin` command.

# Running

After booting the kernel you should see all LEDs are turned on and a terminal is running on PB6/PB7 pins.

The following commands are supported:
- `hi` - says hello
- `+3`/`-3`/`+4`/`-4`/`+5`/`-5`/`+6`/`-6` - turn on/off LD3/4/5/6

# Issues

If you have any issues or questions with the bkernel, just [open and issue](https://github.com/rasendubi/bkernel/issues) or mail me at [rasen.dubi@gmail.com](mailto:rasen.dubi@gmail.com).

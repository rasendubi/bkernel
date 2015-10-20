CC := arm-none-eabi-gcc
LD := arm-none-eabi-gcc
OBJCOPY := arm-none-eabi-objcopy
RUST := rustc

CFLAGS := -std=c99 -pedantic -Wall -Wextra -mcpu=cortex-m4 -msoft-float -nostdlib -lnosys \
	-fPIC -mapcs-frame -ffreestanding -O3 -mthumb-interwork -mlittle-endian -mthumb
LDFLAGS := -N -nostdlib -T stm32_flash.ld
RUSTFLAGS := -g -Z no-landing-pads --target thumbv7em-none-eabi -C opt-level=2 -L lib/thumbv7em-none-eabi

RUSTDIR ?= rust

RUSTC_COMMIT := $(shell rustc -Vv | sed -n 's/^commit-hash: \(.*\)$$/\1/p')

.PHONY: all
all: kernel_meta doc test

.PHONY: kernel_meta
kernel_meta: checkout_rust kernel.bin lib/thumbv7em-none-eabi

kernel.bin: kernel.elf
	$(OBJCOPY) -O binary $^ $@

kernel.elf: src/bootstrap.o src/kernel.o
	$(LD) $(LDFLAGS) -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

src/kernel.o: $(shell find src/ -type f -name '*.rs') lib/thumbv7em-none-eabi/libcore.rlib
	$(RUST) $(RUSTFLAGS) src/kernel.rs -C lto --emit=obj -o $@

lib/thumbv7em-none-eabi/libcore.rlib: $(RUSTDIR)/src/libcore | checkout_rust lib/thumbv7em-none-eabi
	$(RUST) $(RUSTFLAGS) $(RUSTDIR)/src/libcore/lib.rs --out-dir lib/thumbv7em-none-eabi/

lib/thumbv7em-none-eabi:
	mkdir -p $@

rust:
	git clone https://github.com/rust-lang/rust

rust/src/libcore: rust

.PHONY: checkout_rust
checkout_rust: $(RUSTDIR)
	cd $(RUSTDIR) && [ "$$(git rev-parse HEAD)" = "$(RUSTC_COMMIT)" ] || git checkout -q $(RUSTC_COMMIT)

src/libkernel.rlib: $(shell find src/ -type f -name '*.rs') lib/thumbv7em-none-eabi/libcore.rlib
	$(RUST) --crate-type=lib --cfg=doc src/kernel.rs

doc: src/libkernel.rlib
	rustdoc --no-defaults --passes collapse-docs --passes unindent-comments --passes strip-hidden src/kernel.rs --target thumbv7em-none-eabi -L .

.PHONY: test
test: src/libkernel.rlib
	rustdoc src/kernel.rs --test -L .

.PHONY: clean
clean:
	rm -rf *.o *.elf *.bin lib

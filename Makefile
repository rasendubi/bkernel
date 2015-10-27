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
kernel_meta: checkout_rust kernel.bin lib/thumbv7em-none-eabi lib/host

kernel.bin: kernel.elf
	$(OBJCOPY) -O binary $^ $@

kernel.elf: src/bootstrap.o src/kernel.o
	$(LD) $(LDFLAGS) -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

src/kernel.o: $(shell find src/ -type f -name '*.rs') lib/thumbv7em-none-eabi/libcore.rlib \
		lib/thumbv7em-none-eabi/libstm32f4.rlib
	$(RUST) $(RUSTFLAGS) src/kernel.rs -C lto --emit=obj -o $@ --extern stm32f4=lib/thumbv7em-none-eabi/libstm32f4.rlib

lib/thumbv7em-none-eabi/libkernel.rlib: $(shell find src/ -type f -name '*.rs') lib/host/libstm32f4.rlib
	$(RUST) --crate-type=lib src/kernel.rs -L lib/host --out-dir lib/host

lib/host/libkernel.rlib: $(shell find src/ -type f -name '*.rs') lib/host/libstm32f4.rlib
	$(RUST) --crate-type=lib src/kernel.rs -L lib/host --out-dir lib/host

lib/thumbv7em-none-eabi/libstm32f4.rlib: $(shell find stm32f4/ -type f -name '*.rs') lib/thumbv7em-none-eabi/libcore.rlib | lib/thumbv7em-none-eabi
	$(RUST) $(RUSTFLAGS) stm32f4/lib.rs --out-dir lib/thumbv7em-none-eabi/

lib/host/libstm32f4.rlib: $(shell find stm32f4/ -type f -name '*.rs') | lib/host
	$(RUST) stm32f4/lib.rs --out-dir lib/host/

lib/thumbv7em-none-eabi/libcore.rlib: $(RUSTDIR)/src/libcore | checkout_rust lib/thumbv7em-none-eabi
	$(RUST) $(RUSTFLAGS) $(RUSTDIR)/src/libcore/lib.rs --out-dir lib/thumbv7em-none-eabi/

lib/thumbv7em-none-eabi lib/host:
	mkdir -p $@

rust:
	git clone https://github.com/rust-lang/rust

rust/src/libcore: rust

.PHONY: checkout_rust
checkout_rust: $(RUSTDIR)
	cd $(RUSTDIR) && [ "$$(git rev-parse HEAD)" = "$(RUSTC_COMMIT)" ] || git checkout -q $(RUSTC_COMMIT)

doc: doc/kernel doc/stm32f4

doc/kernel: lib/thumbv7em-none-eabi/libcore.rlib lib/thumbv7em-none-eabi/libstm32f4.rlib $(shell find src/ -type f -name '*.rs')
	rustdoc src/kernel.rs --target thumbv7em-none-eabi -L lib/thumbv7em-none-eabi/

doc/stm32f4: lib/thumbv7em-none-eabi/libcore.rlib $(shell find stm32f4/ -type f -name '*.rs')
	rustdoc stm32f4/lib.rs --target thumbv7em-none-eabi -L lib/thumbv7em-none-eabi/

.PHONY: test
test: test_kernel test_stm32f4

.PHONY: test_kernel
test_kernel: lib/host/libkernel.rlib
	rustdoc src/kernel.rs --test -L lib/host

.PHONY: test_stm32f4
test_stm32f4: lib/host/libkernel.rlib
	rustdoc stm32f4/lib.rs --test -L lib/host

.PHONY: clean
clean:
	rm -rf *.o *.elf *.bin lib doc

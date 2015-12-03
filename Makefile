CC := arm-none-eabi-gcc
LD := arm-none-eabi-gcc
OBJCOPY := arm-none-eabi-objcopy
RUST := rustc

TARGET := thumbv7em-none-eabi

CFLAGS := -std=c99 -pedantic -Wall -Wextra -mcpu=cortex-m4 -msoft-float -nostdlib -lnosys \
	-fPIC -mapcs-frame -ffreestanding -O3 -mlittle-endian -mthumb
LDFLAGS := -N -nostdlib -T stm32_flash.ld -Wl,--gc-sections
RUSTFLAGS := -g -Z no-landing-pads --target $(TARGET) -C opt-level=3 -L lib/$(TARGET)

RUSTDIR ?= rust

RUSTC_COMMIT := $(shell rustc -Vv | sed -n 's/^commit-hash: \(.*\)$$/\1/p')

SOURCES := $(shell find src/ stm32f4/ smalloc/ -type f -name '*.rs')

.PHONY: all
all: build doc test
	du -b kernel.bin

.PHONY: build
build: checkout_rust kernel.bin lib/$(TARGET)

kernel.bin: kernel.elf
	$(OBJCOPY) -O binary $^ $@

kernel.elf: target/$(TARGET)/release/libkernel.a stm32_flash.ld
	$(LD) $(LDFLAGS) -o $@ target/$(TARGET)/release/libkernel.a

target/$(TARGET)/release/libkernel.a: $(SOURCES) lib/$(TARGET)/libcore.rlib
	cargo rustc --target=thumbv7em-none-eabi --release -- -Z no-landing-pads

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

lib/$(TARGET)/libcore.rlib: $(RUSTDIR)/src/libcore | checkout_rust lib/$(TARGET)
	$(RUST) $(RUSTFLAGS) $(RUSTDIR)/src/libcore/lib.rs --out-dir lib/$(TARGET)

lib/$(TARGET):
	mkdir -p $@

rust:
	git clone https://github.com/rust-lang/rust

rust/src/libcore: rust

.PHONY: checkout_rust
checkout_rust: $(RUSTDIR)
	cd $(RUSTDIR) && [ "$$(git rev-parse HEAD)" = "$(RUSTC_COMMIT)" ] || git checkout -q $(RUSTC_COMMIT) || ( git fetch && git checkout -q $(RUSTC_COMMIT) )

.PHONY: doc
doc: lib/$(TARGET)/libcore.rlib
	# Cargo doesn't pass custom link directory to `cargo doc`,
	# so building doc for thumbv7em-none-eabi with cargo is impossible now.
	# See https://github.com/rust-lang/cargo/issues/2175
	cargo doc #--target=$(TARGET)

.PHONY: test
test:
	cargo test -p bkernel -p stm32f4 -p smalloc

.PHONY: clean
clean:
	rm -rf *.o *.elf *.bin lib src/*.o
	cargo clean

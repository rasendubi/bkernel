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
DEVICE ?= /dev/ttyUSB0

RUSTC_COMMIT := $(shell rustc -Vv | sed -n 's/^commit-hash: \(.*\)$$/\1/p')

SOURCES := $(shell find src/ stm32f4/ smalloc/ linkmem/ -type f -name '*.rs')
LD_SOURCES := $(wildcard *.ld)

.PHONY: all
all: build doc test
	du -b kernel.bin

.PHONY: build
build: checkout_rust kernel.bin lib/$(TARGET)

kernel.bin: kernel.elf
	$(OBJCOPY) -O binary $^ $@

kernel.elf: target/$(TARGET)/release/libkernel.a $(LD_SOURCES)
	$(LD) $(LDFLAGS) -o $@ target/$(TARGET)/release/libkernel.a

target/$(TARGET)/release/libkernel.a: $(SOURCES) lib/$(TARGET)/libcore.rlib lib/$(TARGET)/liballoc.rlib lib/$(TARGET)/libcollections.rlib
	cargo rustc --target=thumbv7em-none-eabi --release -- -Z no-landing-pads

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

lib/$(TARGET)/libcore.rlib: $(RUSTDIR)/src/libcore | checkout_rust lib/$(TARGET)
	$(RUST) $(RUSTFLAGS) $(RUSTDIR)/src/libcore/lib.rs --out-dir lib/$(TARGET)

lib/$(TARGET)/liballoc.rlib: $(RUSTDIR)/src/liballoc lib/$(TARGET)/libcore.rlib | checkout_rust lib/$(TARGET)
	$(RUST) $(RUSTFLAGS) $(RUSTDIR)/src/liballoc/lib.rs --out-dir lib/$(TARGET)

lib/$(TARGET)/libcollections.rlib: $(RUSTDIR)/src/libcollections lib/$(TARGET)/liballoc.rlib lib/$(TARGET)/libcore.rlib lib/$(TARGET)/librustc_unicode.rlib | checkout_rust lib/$(TARGET)
	$(RUST) $(RUSTFLAGS) $(RUSTDIR)/src/libcollections/lib.rs --out-dir lib/$(TARGET)

lib/$(TARGET)/librustc_unicode.rlib: $(RUSTDIR)/src/librustc_unicode lib/$(TARGET)/libcore.rlib | checkout_rust lib/$(TARGET)
	$(RUST) $(RUSTFLAGS) $(RUSTDIR)/src/librustc_unicode/lib.rs --out-dir lib/$(TARGET)

lib/$(TARGET):
	mkdir -p $@

rust:
	git clone https://github.com/rust-lang/rust

rust/src/libcore: rust
rust/src/liballoc: rust
rust/src/libcollections: rust
rust/src/librustc_unicode: rust

.PHONY: checkout_rust
checkout_rust: $(RUSTDIR)
	cd $(RUSTDIR) && [ "$$(git rev-parse HEAD)" = "$(RUSTC_COMMIT)" ] || git checkout -q $(RUSTC_COMMIT) || ( git fetch && git checkout -q $(RUSTC_COMMIT) )

# Cargo doesn't pass custom link directory to `cargo doc`,
# so building doc for thumbv7em-none-eabi with cargo is impossible now.
# See https://github.com/rust-lang/cargo/issues/2175
.PHONY: doc
doc: # lib/$(TARGET)/libcore.rlib
	cargo doc #--target=$(TARGET)

.PHONY: test
test:
	cargo test -p bkernel -p stm32f4 -p smalloc -p bscheduler

.PHONY: flash
flash: kernel.bin
	openocd -f openocd.cfg -c 'flash_bkernel kernel.bin; exit'

.PHONY: reset
reset:
	openocd -f openocd.cfg -c 'reset; exit'

.PHONY: device_test
device_test:
	expect tests/test.exp $(DEVICE)

.PHONY: clean
clean:
	rm -rf *.o *.elf *.bin lib src/*.o
	cargo clean

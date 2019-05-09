TARGET := thumbv7em-none-eabi

BOARD ?= stm32f407-discovery

RUSTCFLAGS := ${RUSTCFLAGS} -g --target $(TARGET) -C opt-level=3

CLIPPYFLAGS := \
	-A unknown_lints \
	-A char_lit_as_u8 \
	-A inline_always \
	-A identity_op \
	-A doc_markdown \
	-A empty_loop \
	-W cast_possible_wrap \
	-W cast_sign_loss \
	-W float_arithmetic \
	-W non_ascii_literal \
	-W nonminimal_bool \
	-W result_unwrap_used \
	-W shadow_unrelated \
	-W similar_names \
	-W unseparated_literal_suffix \
	-W used_underscore_binding \
	-W wrong_pub_self_convention \
	-W cast_possible_truncation \

DEVICE ?= /dev/ttyUSB0

.PHONY: all
all: build doc test
	du -b kernel.bin

.PHONY: build
build: kernel.bin

kernel.bin: target/$(TARGET)/release/bkernel $(LD_SOURCES) # kernel.elf
	$(OBJCOPY) -O binary $< $@

target/$(TARGET)/release/bkernel: $(SOURCES)
	RUSTFLAGS="${RUSTFLAGS}" cargo build --target=$(TARGET) --release

.PHONY: doc
doc:
	RUSTFLAGS="${RUSTFLAGS}" cargo doc --target=$(TARGET)

.PHONY: test
test:
	RUSTFLAGS="${RUSTFLAGS}" cargo test -p bkernel -p stm32f4 -p breactor -p smalloc -p dev

.PHONY: clippy
clippy:
	cargo clean
	RUSTFLAGS="${RUSTFLAGS}" cargo clippy --target=thumbv7em-none-eabi -p stm32f4 -- ${CLIPPYFLAGS}
	RUSTFLAGS="${RUSTFLAGS}" cargo clippy --target=thumbv7em-none-eabi -p breactor -- ${CLIPPYFLAGS}
	RUSTFLAGS="${RUSTFLAGS}" cargo clippy --target=thumbv7em-none-eabi -p smalloc -- ${CLIPPYFLAGS}
	RUSTFLAGS="${RUSTFLAGS}" cargo clippy --target=thumbv7em-none-eabi -p linkmem -- ${CLIPPYFLAGS}
	RUSTFLAGS="${RUSTFLAGS}" cargo clippy --target=thumbv7em-none-eabi -p dev -- ${CLIPPYFLAGS}
	RUSTFLAGS="${RUSTFLAGS}" cargo clippy --target=thumbv7em-none-eabi -p bkernel -- ${CLIPPYFLAGS}

.PHONY: openocd
openocd:
	openocd -f ${BOARD}.cfg -s openocd/

.PHONY: flash
flash: kernel.bin
	openocd -f ${BOARD}.cfg -s openocd/ -c 'flash_bkernel kernel.bin; exit'

.PHONY: reset
reset:
	openocd -f ${BOARD}.cfg -s openocd/ -c 'reset; exit'

.PHONY: device_test
device_test:
	expect tests/test.exp $(DEVICE)

.PHONY: clean
clean:
	rm -rf *.elf *.bin lib
	cargo clean

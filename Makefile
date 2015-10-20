CC := arm-none-eabi-gcc
LD := arm-none-eabi-gcc
OBJCOPY := arm-none-eabi-objcopy
RUST := rustc

CFLAGS := -std=c99 -pedantic -Wall -Wextra -mcpu=cortex-m4 -msoft-float -nostdlib -lnosys \
	-fPIC -mapcs-frame -ffreestanding -O3 -mthumb-interwork -mlittle-endian -mthumb
LDFLAGS := -N -nostdlib -T stm32_flash.ld
RUSTFLAGS := -Z no-landing-pads --target thumbv7em-none-eabi --emit=obj -L . -C lto -C opt-level=2

.PHONY: all
all: kernel.bin doc test

kernel.bin: kernel.elf
	$(OBJCOPY) -O binary $^ $@

kernel.elf: src/bootstrap.o src/kernel.o
	$(LD) $(LDFLAGS) -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

src/kernel.o: $(shell find src/ -type f -name '*.rs')
	$(RUST) $(RUSTFLAGS) src/kernel.rs -o $@

src/libkernel.rlib: $(shell find src/ -type f -name '*.rs')
	$(RUST) --crate-type=lib --cfg=doc src/kernel.rs

doc: src/libkernel.rlib
	rustdoc --no-defaults --passes collapse-docs --passes unindent-comments --passes strip-hidden src/kernel.rs --target thumbv7em-none-eabi -L .

.PHONY:
test: src/libkernel.rlib
	rustdoc src/kernel.rs --test -L .

.PHONY: clean
clean:
	rm -rf *.o *.elf *.bin

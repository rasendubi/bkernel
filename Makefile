CC := arm-none-eabi-gcc
LD := arm-none-eabi-ld
RUST := rustc

CFLAGS := -std=c99 -pedantic -Wall -Wextra -march=armv6 -msoft-float \
	-fPIC -mapcs-frame -ffreestanding -O3
LDFLAGS := -N -Ttext=0x10000 -nostdlib
RUSTFLAGS := -Z no-landing-pads --target arm-none-eabi --emit=obj -L . -C lto -C opt-level=2

QEMU ?= qemu-system-arm

kernel.elf: bootstrap.o kernel.o
	$(LD) $(LDFLAGS) -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

%.o: %.rs
	$(RUST) $(RUSTFLAGS) $^ -o $@

.PHONY: clean
clean:
	rm -rf *.o

.PHONY: run
run: kernel.elf
	$(QEMU) -M versatilepb -cpu arm1176 -nographic -kernel kernel.elf

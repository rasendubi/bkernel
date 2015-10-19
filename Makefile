CC := arm-none-eabi-gcc
LD := arm-none-eabi-gcc
OBJCOPY := arm-none-eabi-objcopy
RUST := rustc

CFLAGS := -std=c99 -pedantic -Wall -Wextra -mcpu=cortex-m4 -msoft-float -nostdlib -lnosys \
	-fPIC -mapcs-frame -ffreestanding -O3 -mthumb-interwork -mlittle-endian -mthumb
LDFLAGS := -N -nostdlib -T stm32_flash.ld
RUSTFLAGS := -Z no-landing-pads --target thumbv7em-none-eabi --emit=obj -L . -C lto -C opt-level=2

QEMU ?= qemu-system-arm

kernel.bin: kernel.elf
	$(OBJCOPY) -O binary $^ $@

kernel.elf: bootstrap.o kernel.o
	$(LD) $(LDFLAGS) -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

kernel.o: $(wildcard *.rs)
	$(RUST) $(RUSTFLAGS) kernel.rs -o $@

.PHONY: clean
clean:
	rm -rf *.o *.elf *.bin

.PHONY: run
run: kernel.elf
	$(QEMU) -M versatilepb -cpu arm1176 -nographic -kernel kernel.elf

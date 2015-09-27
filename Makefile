CC := arm-none-eabi-gcc
LD := arm-none-eabi-ld
NIM := nim

CFLAGS := -std=c99 -pedantic -Wall -Wextra -march=armv6 -msoft-float \
	-fPIC -mapcs-frame -ffreestanding
LDFLAGS := -N -Ttext=0x10000 -nostdlib
NIMFLAGS := --parallelBuild:1 --deadCodeElim:on --gcc.exe:$(CC) \
	--noMain --noLinking --gc:none --cpu:arm --os:standalone \
	--passC:\"$(CFLAGS)\"

QEMU ?= qemu-system-arm

kernel.elf: bootstrap.o nimcache/kernel.o nimcache/stdlib_system.o
	$(LD) $(LDFLAGS) -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -o $@ -c $^

nimcache/%.o: %.nim
	$(NIM) $(NIMFLAGS) c $^

.PHONY: clean
clean:
	rm -rf *.o nimcache

.PHONY: run
run: kernel.elf
	$(QEMU) -M versatilepb -cpu arm1176 -nographic -kernel kernel.elf

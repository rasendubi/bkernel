target remote :3333

set print asm-demangle on

break __isr__default
break rust_begin_unwind

monitor arm semihosting enable

load

stepi
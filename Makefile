TARGET = armv7-unknown-none
OUTPUT = kernel.img
OBJDUMP = cargo objdump -- -D
OBJCOPY = cargo objcopy -- --strip-all -O binary
BUILD = cargo xbuild
ASM_OUTPUT = rusty.asm

release:
	$(BUILD) --release
	$(OBJDUMP) target/$(TARGET)/release/rustybeagle > $(ASM_OUTPUT)
	$(OBJCOPY) target/$(TARGET)/release/rustybeagle $(OUTPUT)
debug:
	$(BUILD)
	$(OBJDUMP) target/$(TARGET)/debug/rustybeagle > $(ASM_OUTPUT)
	$(OBJCOPY) target/$(TARGET)/debug/rustybeagle $(OUTPUT)
minicom:
	minicom -b 115200 -D /dev/ttyUSB0
clean:
	cargo clean
size:
	cargo size target/$(TARGET)/release/rustybeagle

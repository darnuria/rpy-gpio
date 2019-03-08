
TARGET = armv7-unknown-linux-gnueabihf
OBJCOPY = cargo objcopy --
bin_name = gpio
OBJCOPY_PARAMS = --strip-all -O binary

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S)

all: target/$(TARGET)/release/gpio $(SOURCES)

target/$(TARGET)/release/$(bin_name): $(SOURCES)
	cargo build --target=$(TARGET) --release

upload: target/$(TARGET)/release/$(bin_name)
	scp $< pi@192.168.1.68:~/$(bin_name)

.PHONY: all clippy clean objdump


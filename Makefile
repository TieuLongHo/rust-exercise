EXEC    = target/thumbv7em-none-eabihf/release/rtic-display

OBJCOPY = cargo-objcopy
OBJDUMP = cargo-objdump
SIZE    = cargo-size
UF2CONV = uf2conv.py

# Application address, depends on Nordic SoftDevice installed with the boot-
# loader. Adafruit nRF52 Bootloader currently ships with S140 v6.1.1. See also:
# https://infocenter.nordicsemi.com/topic/sds_s140/SDS/s1xx/mem_usage/mem_resource_reqs.html
UF2BASE = 0x26000

.PHONY: all
all: $(EXEC).bin $(EXEC).uf2 $(EXEC).lst

$(EXEC).bin:
	$(OBJCOPY) --release -- -O binary $@
	$(SIZE) --release -- -A

$(EXEC).uf2: $(EXEC).bin
	python $(UF2CONV) -f 0xADA52840 --base $(UF2BASE) --output $@ $<

$(EXEC).lst: $(EXEC)
	$(OBJDUMP) --release -- -D > $@

.PHONY: clean
clean:
	cargo clean

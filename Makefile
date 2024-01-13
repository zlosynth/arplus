.PHONY: all
all: format clippy test

.PHONY: check-format
check-format:
	make -C dsp check-format
	make -C firmware check-format

.PHONY: format
format:
	make -C dsp format
	make -C firmware format

.PHONY: clippy
clippy:
	make -C dsp clippy
	make -C firmware clippy

.PHONY: test
test:
	make -C dsp test

.PHONY: update
update:
	make -C dsp update
	make -C firmware update

.PHONY: clean
clean:
	make -C dsp clean
	make -C firmware clean

.PHONY: flash
flash:
	make -C firmware flash

.PHONY: flash-dfu
flash:
	make -C firmware flash-dfu

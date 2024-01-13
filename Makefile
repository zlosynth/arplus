.PHONY: all
all: format clippy

.PHONY: check-format
check-format:
	make -C firmware check-format

.PHONY: format
format:
	make -C firmware format

.PHONY: clippy
clippy:
	make -C firmware clippy

.PHONY: update
update:
	make -C firmware update

.PHONY: clean
clean:
	make -C firmware clean

.PHONY: flash
flash:
	make -C firmware flash

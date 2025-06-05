CARGO = cargo +1.86.0

.PHONY: all
all: format clippy

.PHONY: check-format
check-format:
	$(CARGO) fmt --all -- --check

.PHONY: format
format:
	$(CARGO) fmt --all

.PHONY: clippy
clippy:
	$(CARGO) clippy --all --tests

.PHONY: update
update:
	$(CARGO) update

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: bloat
bloat:
	$(CARGO) size --bin main --release -- -m
	$(CARGO) bloat --release

.PHONY: flash
flash:
	$(CARGO) run --release --features idle-measuring

.PHONY: flash-dfu
flash-dfu:
	$(CARGO) objcopy --release -- -O binary target/arplus.bin
	dfu-util -a 0 -s 0x08000000:leave -D target/arplus.bin -d ,0483:df11

.PHONY: diagnostics
diagnostics:
	$(CARGO) test --release --test diagnostics


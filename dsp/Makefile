CARGO = cargo

.PHONY: all
all: format clippy test

.PHONY: check-format
check-format:
	$(CARGO) fmt --all -- --check

.PHONY: format
format:
	$(CARGO) fmt --all

.PHONY: clippy
clippy:
	# TODO: $(CARGO) clippy --all -- -D warnings
	$(CARGO) clippy --all

.PHONY: test
test:
ifdef SLOW
	$(CARGO) test -- --ignored
else
	$(CARGO) test
endif

.PHONY: update
update:
	$(CARGO) update

.PHONY: clean
clean:
	$(CARGO) clean

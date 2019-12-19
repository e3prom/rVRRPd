TARGET = target/release
BINARY = rvrrpd-pw
PREFIX = /usr

main:
	@cargo build --release

test:
	@cargo test

check:
	@cargo fmt --all -- --check

clean:
	@cargo clean

install:
	cp $(TARGET)/${BINARY} $(PREFIX)/bin/${BINARY}
	chmod 755 $(PREFIX)/bin/rvrrpd-pw

.PHONY: main test check clean install

TARGET = target/release
BINARY = main
PREFIX = /usr

main:
	@cargo build --release

test:
	@cargo test

docs:
	@cargo doc --no-deps

check:
	@cargo fmt --all -- --check

clean:
	@cargo clean

install:
	cp $(TARGET)/${BINARY} $(PREFIX)/bin/rvrrpd
	chmod 755 $(PREFIX)/bin/rvrrpd
	if [ ! -d "/etc/rvrrpd" ]; then \
		mkdir /etc/rvrrpd; \
	fi

.PHONY: main test docs check clean install
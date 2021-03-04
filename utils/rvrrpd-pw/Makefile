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
	if [ ! -d $(DESTDIR)$(PREFIX)/bin ]; then \
		 mkdir -p $(DESTDIR)$(PREFIX)/bin; \
	fi
	cp $(TARGET)/${BINARY} $(DESTDIR)$(PREFIX)/bin/${BINARY}
	chmod 755 $(DESTDIR)$(PREFIX)/bin/${BINARY}

.PHONY: main test check clean install

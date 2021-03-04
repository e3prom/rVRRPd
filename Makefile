TARGET = target/release
BINARY = rvrrpd
PREFIX = /usr

main: rvrrpd-pw
	@cargo build --release

test:
	@cargo test

docs:
	@cargo doc --no-deps

check:
	@cargo fmt --all -- --check

clean: rvrrpd-pw-clean
	@cargo clean

install: rvrrpd-pw-install
	if [ ! -d $(DESTDIR)$(PREFIX)/sbin ]; then \
		mkdir -p $(DESTDIR)$(PREFIX)/sbin; \
	fi
	cp $(TARGET)/${BINARY} $(DESTDIR)$(PREFIX)/sbin/${BINARY}
	chmod 755 $(DESTDIR)$(PREFIX)/sbin/${BINARY}
	if [ ! -d $(DESTDIR)/etc/rvrrpd ]; then \
		mkdir -p $(DESTDIR)/etc/rvrrpd; \
	fi

rvrrpd-pw:
	cd utils/rvrrpd-pw && $(MAKE)

rvrrpd-pw-install:
	cd utils/rvrrpd-pw && $(MAKE) install

rvrrpd-pw-clean:
	cd utils/rvrrpd-pw && $(MAKE) clean

.PHONY: main test docs check clean install

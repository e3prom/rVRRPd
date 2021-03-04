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
	[ ! -d $(DESTDIR)$(PREFIX)/sbin ] && mkdir -p $(DESTDIR)$(PREFIX)/sbin
	cp $(TARGET)/${BINARY} $(DESTDIR)$(PREFIX)/sbin/${BINARY}
	chmod 755 $(DESTDIR)$(PREFIX)/sbin/${BINARY}
	[ ! -d $(DESTDIR)/etc/rvrrpd ] && mkdir -p $(DESTDIR)/etc/rvrrpd

rvrrpd-pw:
	$(MAKE) -C utils/rvrrpd-pw

rvrrpd-pw-install:
	$(MAKE) -C utils/rvrrpd-pw install

rvrrpd-pw-clean:
	$(MAKE) -C utils/rvrrpd-pw clean

.PHONY: main test docs check clean install

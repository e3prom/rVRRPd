TARGET = target/release
BINARY = rvrrpd
PREFIX = /usr

all: main ${BINARY}.8.gz

main: rvrrpd-pw
	@cargo build --release

test:
	@cargo test

docs:
	@cargo doc --no-deps

check:
	@cargo fmt --all -- --check

${BINARY}.8.gz: main
	help2man -N -s8 -n'lightweight, fast, and highly secure VRRP daemon' $(TARGET)/${BINARY} | gzip > $@

clean: rvrrpd-pw-clean
	rm -f ${BINARY}.8.gz
	@cargo clean

install: rvrrpd-pw-install
	if [ ! -d $(DESTDIR)$(PREFIX)/sbin ]; then \
		mkdir -p $(DESTDIR)$(PREFIX)/sbin; \
	fi
	cp $(TARGET)/${BINARY} $(DESTDIR)$(PREFIX)/sbin/${BINARY}
	chmod 755 $(DESTDIR)$(PREFIX)/sbin/${BINARY}
	cp ${BINARY}.8.gz $(DESTDIR)$(PREFIX)/share/man/man8/${BINARY}.8.gz
	if [ ! -d $(DESTDIR)/etc/rvrrpd ]; then \
		mkdir -p $(DESTDIR)/etc/rvrrpd; \
	fi

rvrrpd-pw:
	$(MAKE) -C utils/rvrrpd-pw

rvrrpd-pw-install:
	$(MAKE) -C utils/rvrrpd-pw install

rvrrpd-pw-clean:
	$(MAKE) -C utils/rvrrpd-pw clean

.PHONY: main test docs check clean install

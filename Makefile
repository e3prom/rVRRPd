TARGET = target/release
BINARY = main
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
	cp $(TARGET)/${BINARY} $(PREFIX)/sbin/rvrrpd
	chmod 755 $(PREFIX)/sbin/rvrrpd
	if [ ! -d "/etc/rvrrpd" ]; then \
		mkdir /etc/rvrrpd; \
	fi

rvrrpd-pw:
	cd utils/rvrrpd-pw && $(MAKE)

rvrrpd-pw-install:
	cd utils/rvrrpd-pw && $(MAKE) install

rvrrpd-pw-clean:
	cd utils/rvrrpd-pw && $(MAKE) clean

.PHONY: main test docs check clean install

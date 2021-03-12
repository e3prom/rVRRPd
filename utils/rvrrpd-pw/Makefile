TARGET = target/release
BINARY = rvrrpd-pw
PREFIX = /usr

all: main ${BINARY}.1.gz

main:
	@cargo build --release

test:
	@cargo test

check:
	@cargo fmt --all -- --check

${BINARY}.1.gz: main
	help2man -N -s1 -n'quick and easy password generation for rVRRPd' $(TARGET)/${BINARY} | gzip > $@

clean:
	rm -f ${BINARY}.1.gz
	@cargo clean

install:
	if [ ! -d $(DESTDIR)$(PREFIX)/bin ]; then \
		 mkdir -p $(DESTDIR)$(PREFIX)/bin; \
	fi
	cp $(TARGET)/${BINARY} $(DESTDIR)$(PREFIX)/bin/${BINARY}
	chmod 755 $(DESTDIR)$(PREFIX)/bin/${BINARY}
	cp ${BINARY}.1.gz $(DESTDIR)$(PREFIX)/share/man/man1/${BINARY}.1.gz

.PHONY: main test check clean install

PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin

TARGET = release
BINARY = target/$(TARGET)/omarchy-circle-search

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --$(TARGET)

install: build
	install -Dm755 $(BINARY) $(DESTDIR)$(BINDIR)/omarchy-circle-search

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/omarchy-circle-search

clean:
	cargo clean

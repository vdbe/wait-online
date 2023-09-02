prefix ?= /usr/local
bindir = $(prefix)/bin
libdir = $(prefix)/lib

TARGET = debug
DEBUG ?= 0

ARGS ?= --no-default-features

ifeq ($(DEBUG),0)
	TARGET = release
	ARGS += --release
endif

BINARY = target/${TARGET}/wait-online
SERVICE =  standalone-network-wait-online.service

SOURCES = $(shell find src -type f -name '*.rs') build.rs Cargo.toml Cargo.lock

all: $(BINARY)

clean:
	cargo clean

distclean: clean
	rm -rf .cargo

## Building the binaries

bin $(BINARY): $(SOURCES)
	env prefix=${prefix} \
		cargo build $(ARGS)

## Install commands

install: install-bin install-service

install-bin: ${BINARY}
	install -Dm0755 "$(BINARY)" "$(DESTDIR)$(bindir)/wait-online"

install-service: ${BINARY}
	install -Dm0644 "target/$(SERVICE)" "$(DESTDIR)$(libdir)/systemd/system/$(SERVICE)"

## Uninstall Commands

uninstall: uninstall-service uninstall-bin

uninstall-bin:
	rm "$(DESTDIR)$(bindir)/wait-online"

uninstall-service:
	rm "$(DESTDIR)$(libdir)/systemd/system/$(SERVICE)"

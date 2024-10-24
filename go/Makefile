VIRTLINT_BASE = $(shell realpath ../target/*/debug 2>/dev/null)
#some dummy fallback
ifeq ($(VIRTLINT_BASE),)
VIRTLINT_BASE := $(PWD)/target/x86_64-unknown-linux-gnu/debug/
endif

OS = $(shell uname -s)

all: libvirt-mod virt-lint-cli

libvirt-mod:
	go mod download libvirt.org/go/libvirt

virt-lint-cli: virt-lint-cli.go virt-lint/virt-lint.go $(VIRTLINT_BASE)/virt_lint.h
ifeq ($(OS), Darwin)
	DYLD_LIBRARY_PATH="$(VIRTLINT_BASE)" PKG_CONFIG_PATH=$(VIRTLINT_BASE) go build $<
else
	LD_LIBRARY_PATH="$(VIRTLINT_BASE)" PKG_CONFIG_PATH=$(VIRTLINT_BASE) go build $<
endif

clean:
	rm -f virt-lint-cli

test: all
ifeq ($(OS), Darwin)
	DYLD_LIBRARY_PATH="$(VIRTLINT_BASE)" PKG_CONFIG_PATH=$(VIRTLINT_BASE) go test virt-lint/*
else
	LD_LIBRARY_PATH="$(VIRTLINT_BASE)" PKG_CONFIG_PATH=$(VIRTLINT_BASE) go test virt-lint/*
endif

fmt:
	go fmt
	go fmt virt-lint/*.go

vet:
	go vet
	go vet virt-lint/*.go

.PHONY: run
run: all
ifeq ($(OS), Darwin)
	DYLD_LIBRARY_PATH="$(VIRTLINT_BASE)" ./virt-lint-cli
else
	LD_LIBRARY_PATH="$(VIRTLINT_BASE)" ./virt-lint-cli
endif

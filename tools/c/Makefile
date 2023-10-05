VIRTLINT_BASE = $(shell realpath ../../target/*/debug 2>/dev/null)
#some dummy fallback
ifeq ($(VIRTLINT_BASE),)
VIRTLINT_BASE := $(PWD)/target/x86_64-unknown-linux-gnu/debug/
endif
INCLUDE = $(shell PKG_CONFIG_PATH=$(VIRTLINT_BASE) pkg-config --cflags --libs virt_lint-uninstalled libvirt)

all: virt-lint

virt-lint: virt-lint.c $(VIRTLINT_BASE)/virt_lint.h
	$(CC) $(CFLAGS) $(INCLUDE) $< -o $@

clean:
	rm -f virt-lint

.PHONY: run
run: all
	LD_LIBRARY_PATH="$(VIRTLINT_BASE)" ./virt-lint

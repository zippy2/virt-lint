VERSION ?= $(shell git describe --tags HEAD 2>/dev/null || echo "0.0.1")
PYTAG = $(shell python -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')
prefix ?= /usr
datarootdir ?= $(prefix)/share

all: rust c-build go-build

# Unfortunately, we have to create the symlink ourselves:
#
# https://github.com/lu-zero/cargo-c/issues/345
#
# After that, we can enable package.metadata.capi.library.versioning.
rust: rust-build rust-cbuild

rust-build:
	cargo build
	pushd target/debug/ && ln -sf libvirt_lint_python.so virt_lint$(PYTAG); \
	popd

rust-cbuild:
	cargo cbuild --prefix="/usr" --libdir="/usr/lib64" --manifest-path=src/Cargo.toml
	pushd target/*/debug/ && ln -sf libvirt_lint.so libvirt_lint.so.0 && \
		ln -sf libvirt_lint.so libvirt_lint.so.0.0.1 ;  \
	popd

rust-check: rust
	cargo test

c-build: rust
	$(MAKE) -C tools/c/

c-run: c-build
	$(MAKE) -C tools/c/ run

go-build: rust-cbuild
	$(MAKE) -C go/

go-run: go-build
	$(MAKE) -C go/ run

go-test:
	$(MAKE) -C go/ test

python-run: rust-build
	$(MAKE) -C python/ run

clean:
	cargo clean
	rm -f virt-lint-$(VERSION).tar.xz virt-lint-$(VERSION)-vendor.tar.xz
	$(MAKE) -C tools/c/ clean
	$(MAKE) -C go/ clean

check: rust-check go-test

fmt:
	cargo fmt
	$(MAKE) -C go/ fmt

dist: virt-lint-$(VERSION).tar.xz

virt-lint-$(VERSION).tar.xz:
	@rm -rf virt-lint-$(VERSION) && \
	mkdir virt-lint-$(VERSION) && \
	cp --parents `git ls-files` virt-lint-$(VERSION) && \
	tar -cJf virt-lint-$(VERSION).tar.xz virt-lint-$(VERSION) && \
	echo Created $@; \
	rm -rf virt-lint-$(VERSION)

virt-lint-$(VERSION)-vendor.tar.xz:
	@cargo vendor && \
	tar -cJf $@ vendor/ && \
	echo Created $@; \
	rm -rf vendor

rpm: virt-lint-$(VERSION).tar.xz virt-lint-$(VERSION)-vendor.tar.xz
	mkdir -p ~/rpmbuild/SOURCES && \
	cp virt-lint-$(VERSION).tar.xz ~/rpmbuild/SOURCES && \
	cp virt-lint-$(VERSION)-vendor.tar.xz ~/rpmbuild/SOURCES && \
	rpmbuild -ba virt-lint.spec && \
	rm -f virt-lint-$(VERSION)-vendor.tar.xz

install-data:
	mkdir -p $(DESTDIR)$(datarootdir)/virt-lint/validators_lua
	cp --recursive validators_lua $(DESTDIR)$(datarootdir)/virt-lint/
	mkdir -p $(DESTDIR)$(datarootdir)/virt-lint/validators_python
	cp --recursive validators_python $(DESTDIR)$(datarootdir)/virt-lint/

uninstall-data:
	rm -rf $(DESTDIR)$(datarootdir)/virt-lint/

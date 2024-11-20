# Ideally, this would be rust2rpm generated with a very few modification, but
# unfortunately, we have to go further. Firstly, Fedora explicitly forbids
# vendoring but not all crates we need are packaged. Secondly, we don't really
# need to package the Rust library itself, rather than its C bindings. And
# rust2rpm is in no way prepared for that. Hence the hackish nature of the
# whole spec file.

# Don't produce debug packages
%global debug_package %{nil}

%global crate virt-lint

Name:           virt-lint
Version:        0.0.1
Release:        %autorelease
Summary:        Virtualization linting library

License:        LGPL-3.0-or-later
URL:            https://gitlab.com/MichalPrivoznik/virt-lint
Source0:        virt-lint-0.0.1.tar.xz
Source1:        virt-lint-0.0.1-vendor.tar.xz

BuildRequires:  rust-packaging
BuildRequires:  cargo-c
BuildRequires:  pkgconfig(libvirt)
BuildRequires:  pkgconfig(libxml-2.0)
BuildRequires:  pkgconfig(lua)
BuildRequires:  pkgconfig(python3)

%global _description %{expand:
%{summary}.}

%description %{_description}

%prep
%autosetup -n %{crate}-%{version} -p1
%cargo_prep
# Now fix up cargo config to allow vendoring and unpack the vendored archive
(
echo '
[build]
rustc = "/usr/bin/rustc"
rustdoc = "/usr/bin/rustdoc"

[profile.rpm]
inherits = "release"
opt-level = 3
codegen-units = 1
debug = 2
strip = "none"

[env]
CFLAGS = "-O2 -flto=auto -ffat-lto-objects -fexceptions -g -grecord-gcc-switches -pipe -Wall -Werror=format-security -Werror=implicit-function-declaration -Werror=implicit-int -Wp,-U_FORTIFY_SOURCE,-D_FORTIFY_SOURCE=3 -Wp,-D_GLIBCXX_ASSERTIONS -specs=/usr/lib/rpm/redhat/redhat-hardened-cc1 -fstack-protector-strong -specs=/usr/lib/rpm/redhat/redhat-annobin-cc1  -m64   -mtune=generic -fasynchronous-unwind-tables -fstack-clash-protection -fcf-protection -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer "
CXXFLAGS = "-O2 -flto=auto -ffat-lto-objects -fexceptions -g -grecord-gcc-switches -pipe -Wall -Werror=format-security -Wp,-U_FORTIFY_SOURCE,-D_FORTIFY_SOURCE=3 -Wp,-D_GLIBCXX_ASSERTIONS -specs=/usr/lib/rpm/redhat/redhat-hardened-cc1 -fstack-protector-strong -specs=/usr/lib/rpm/redhat/redhat-annobin-cc1  -m64   -mtune=generic -fasynchronous-unwind-tables -fstack-clash-protection -fcf-protection -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer "
LDFLAGS = "-Wl,-z,relro -Wl,--as-needed  -Wl,-z,now -specs=/usr/lib/rpm/redhat/redhat-hardened-ld -specs=/usr/lib/rpm/redhat/redhat-annobin-cc1  -Wl,--build-id=sha1 -specs=/usr/lib/rpm/redhat/redhat-package-notes "

[term]
verbose = true

[source.local-registry]
directory = "/usr/share/cargo/registry"

[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "./vendor"
' > .cargo/config

tar -xoaf %{SOURCE1}
)

%build
# And since rpmbuild removes builddir in between %%build and %%install, make this
# NOP and do the compilation in %%install.
#CARGO_HOME=".cargo" cargo cbuild

%install
CARGO_HOME=".cargo" cargo cinstall --destdir=%{buildroot} --prefix=%{_prefix} --libdir=%{_libdir} --manifest-path=src/Cargo.toml
make install-data DESTDIR=%{buildroot} prefix=%{_prefix}

%files
%{_includedir}/virt_lint/virt_lint.h
%{_libdir}/libvirt_lint.so*
%{_libdir}/libvirt_lint.a
%{_libdir}/pkgconfig/virt_lint.pc
%{_datadir}/virt-lint/validators_lua
%{_datadir}/virt-lint/validators_python

%changelog
%autochangelog

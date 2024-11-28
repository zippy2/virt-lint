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
Release:        1%{?dist}
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
Requires: python3-libvirt

%global _description %{expand:
%{summary}.}

%description %{_description}

%package devel
Summary: Libraries, includes, etc. to compile with virt-lint
Requires: virt-lint = %{version}-%{release}

%description devel
Include header files & development libraries for the virt-lint library.

%package validators-lua
Summary: Validators written in Lua for the virt-lint library.
Requires: virt-lint = %{version}-%{release}

%description validators-lua
Validators written in Lua for the virt-lint library.

%package validators-python
Summary: Validators written in Python for the virt-lint library.
Requires: virt-lint = %{version}-%{release}

%description validators-python
Validators written in Python for the virt-lint library.

%prep
%autosetup -n %{crate}-%{version} -p1
%cargo_prep
# Now fix up cargo config to allow vendoring and unpack the vendored archive
(
sed -i 's,/usr/share/cargo/registry,./vendor,' .cargo/config.toml
tar -xoaf %{SOURCE1}
)

%build
cd src/
%cargo_cbuild

%install
cd src/
%cargo_cinstall
cd ..
make install-data DESTDIR=%{buildroot} prefix=%{_prefix}

%files
%{_libdir}/libvirt_lint.so.*

%files devel
%{_libdir}/libvirt_lint.so
%{_includedir}/virt_lint/virt_lint.h
%{_libdir}/libvirt_lint.a
%{_libdir}/pkgconfig/virt_lint.pc

%files validators-lua
%{_datadir}/virt-lint/validators_lua

%files validators-python
%{_datadir}/virt-lint/validators_python

%changelog
%autochangelog

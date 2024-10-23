# Copyright 2023-2024 Gentoo Authors
# Distributed under the terms of the GNU General Public License v2

# Auto-Generated by cargo-ebuild 0.5.4-r1

EAPI=8

if [[ ${PV} != *9999 ]]; then
CRATES="
	anstream@0.6.15
	anstyle@1.0.8
	anstyle-parse@0.2.5
	anstyle-query@1.1.1
	anstyle-wincon@3.0.4
	autocfg@1.4.0
	bstr@1.10.0
	cc@1.1.30
	cfg-if@1.0.0
	clap@4.5.20
	clap_builder@4.5.20
	clap_derive@4.5.18
	clap_lex@0.7.2
	colorchoice@1.0.2
	enum-display-derive@0.1.1
	heck@0.5.0
	is_terminal_polyfill@1.70.1
	libc@0.2.159
	libxml@0.3.3
	memchr@2.7.4
	mlua@0.9.9
	mlua-sys@0.6.3
	num-traits@0.2.19
	once_cell@1.20.2
	pkg-config@0.3.31
	pkg-version@1.0.0
	pkg-version-impl@0.1.1
	proc-macro-hack@0.5.20+deprecated
	proc-macro2@1.0.87
	quote@1.0.37
	rustc-hash@2.0.0
	serde@1.0.210
	serde_derive@1.0.210
	shlex@1.3.0
	strsim@0.11.1
	syn@1.0.109
	syn@2.0.79
	thiserror@1.0.64
	thiserror-impl@1.0.64
	unicode-ident@1.0.13
	utf8parse@0.2.2
	uuid@1.10.0
	vcpkg@0.2.15
	virt@0.4.1
	virt-sys@0.3.0
	windows-sys@0.52.0
	windows-targets@0.52.6
	windows_aarch64_gnullvm@0.52.6
	windows_aarch64_msvc@0.52.6
	windows_i686_gnu@0.52.6
	windows_i686_gnullvm@0.52.6
	windows_i686_msvc@0.52.6
	windows_x86_64_gnu@0.52.6
	windows_x86_64_gnullvm@0.52.6
	windows_x86_64_msvc@0.52.6
"
fi

inherit edo cargo go-module

DESCRIPTION="Virtualization linting library"
HOMEPAGE="https://gitlab.com/MichalPrivoznik/virt-lint"

if [[ ${PV} == *9999 ]] ; then
	EGIT_REPO_URI="https://gitlab.com/MichalPrivoznik/virt-lint"
	inherit git-r3
else
	SRC_URI="https://gitlab.com/MichalPrivoznik/virt-lint/-/archive/v${PV}/libvirt-v${PV}.tar.bz2 -> ${P}.tar.bz2"
	SRC_URI+="${CARGO_CRATE_URIS}"
	KEYWORDS="~amd64 ~x86"
fi

# License set may be more restrictive as OR is not respected
# use cargo-license for a more accurate license picture
LICENSE="0BSD Apache-2.0 LGPL-2.1 LGPL-3+ MIT Unicode-DFS-2016 Unlicense"
SLOT="0"
IUSE="+c +go static-libs"
REQUIRED_USE="
	go? ( c )
	static-libs? ( c )
"

DEPEND="
	dev-lang/lua:5.4
	c? ( app-emulation/libvirt )"
RDEPEND="${DEPEND}"
BDEPEND="${RDEPEND}
	c? ( dev-util/cargo-c )
	go? ( dev-lang/go )"

# rust does not use *FLAGS from make.conf, silence portage warning
# update with proper path to binaries this crate installs, omit leading /
QA_FLAGS_IGNORED="usr/bin/${PN}"

src_unpack() {
	if [[ ${PV} == *9999* ]]; then
		git-r3_src_unpack
		cargo_live_src_unpack
	else
		default
		cargo_src_unpack
	fi
}

src_compile() {
	export CARGO_HOME="${ECARGO_HOME}"
	local cargoargs=(
		--manifest-path=src/Cargo.toml
		--library-type=cdylib
		--prefix=/usr
		--libdir="/usr/$(get_libdir)"
		$(usev !debug '--release')
	)

	cargo_src_compile

	if use c
	then
		edo cargo cbuild "${cargoargs[@]}"
		use static-libs && edo cargo cbuild --library-type=staticlib "${cargoargs[@]}"
	fi
}

src_install() {
	export CARGO_HOME="${ECARGO_HOME}"
	local cargoargs=(
		--manifest-path=src/Cargo.toml
		--library-type=cdylib
		--prefix=/usr
		--libdir="/usr/$(get_libdir)"
		--destdir="${ED}"
		$(usev !debug '--release')
	)

	cargo_src_install --path ./tools

	emake DESTDIR="${D}" install-data

	if use c
	then
		edo cargo cinstall "${cargoargs[@]}"
		edo cargo cinstall --library-type=staticlib "${cargoargs[@]}"
	fi
}

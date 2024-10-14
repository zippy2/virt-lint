# virt-lint

Virtualization linting library written in Rust.
[virt-lint](https://gitlab.com/MichalPrivoznik/virt-lint)

## Building from sources

Cargo handles all the dependencies. Just run:

```shell
cargo build
```

If you want to build `C` language bindings, you will need
[cargo-c](https://github.com/lu-zero/cargo-c). Then you can just run:

```shell
cargo cbuild --manifest-path=src/Cargo.toml
```

There is even a small `C` command line utility that demonstrates use of C
language bindings. To compile it run:

```shell
make -C tools/c/
```

Alternatively, you may just run `make` which bundles all necessary steps.

## Usage

### Rust library API

The Rust API is very easy to consume. Just create a `VirtLint` object. It
accepts one argument (libvirt connection), optionally.  If specified, it will
be used later by validator callbacks. If not specified (None), aka 'offline
mode', then additional APIs must be called to pass previously obtained
information (capabilities, domain capabilities). The object holds a reference
to the connection so the original can be closed.

```rust
let mut vl = VirtLint::new(Some(conn));

if let Err(e) = conn.close() {
    panic!("Failed to disconnect from hypervisor: {}", e);
}
```

When the offline mode is used, then the following APIs can be used to feed
information to validator callbacks:

```rust
// Set capabilities obtained earlier
vl.capabilities_set(Some("<capabilities>...</capabilities"))?;

// Clear previously stored capabilities
vl.capabilities_set(None)?;

// Add domain capabilities
vl.domain_capabilities_add("<domainCapabilities>...</domainCapabilities>")?;

// Clear previously added domain capabilities
vl.domain_capabilities_clear();
```

Both capabilities and domain capabilities XMLs are parsed (hence corresponding
APIs may fail) and cached internally.


Now we are all set to validate a domain XML:

```rust
let validators = Vec::<String>::new();
let error_on_no_connect = false;

if let Err(e) = vl.validate(&domxml, &validators, error_on_no_connect) {
    println!("Validation failed: {}", e);
}
```

Internally, VirtLint has a set of rules which check for erroneous or suboptimal
configuration (as expressed by domain XML). Each rule has a tag (`String`)
associated with it. This allows caller to run only a specified subset of
checks. As a shortcut, if no tags are specified (i.e. an empty vector is
passed), all validation rules are run regardless of their tag.

And finally, we can get list of warning produced by rules:

```rust
for w in l.warnings().iter() {
    let (tags, domain, level, msg) = w.get();
    println!(
        "Warning: tags={:?}\tdomain={domain}\tlevel={level}\tmsg={msg}",
        tags
    );
}
```

To list all available tags, we can call `list_validator_tags()` method:

```rust
VirtLint::list_validator_tags()
    .iter()
    .for_each(|tag| println!("{tag}"));
```

### C library API

The C API is written so that it models Rust API as closely as possible. We are
given constructor and destructor functions:

```c
typedef struct VirtLint VirtLint;

struct VirtLint *virt_lint_new(virConnectPtr conn);

void virt_lint_free(struct VirtLint *vl);
```

The validation function looks also similar to its Rust version:

```c
int virt_lint_validate(struct VirtLint *vl,
                       const char *domxml,
                       const char **tags,
                       size_t ntags,
                       bool error_on_no_connect,
                       struct VirtLintError **err);
```

And so does function obtaining the list of warnings:

```c
typedef struct CVirtLintWarning {
  char **tags;
  size_t ntags;
  enum WarningDomain domain;
  enum WarningLevel level;
  char *msg;
} CVirtLintWarning;

ptrdiff_t virt_lint_get_warnings(const struct VirtLint *vl,
                                 struct CVirtLintWarning **warnings,
                                 struct VirtLintError **err);

void virt_lint_warnings_free(struct CVirtLintWarning **warnings, ptrdiff_t *nwarnings);
```

Please note, `ptrdiff_t` is basically the same as `ssize_t`. It's only that
cargo-c translates `isize` into `ptrdiff_t`.


Anyway, listing tags is also very similar:

```c
ptrdiff_t virt_lint_list_tags(char ***tags, struct VirtLintError **err);

void virt_lint_warnings_free(struct CVirtLintWarning **warnings, ptrdiff_t *nwarnings);
```

Because C program may use different allocator than Rust, we also need additional free functions:

```c
void virt_lint_string_free(char *string);

void virt_lint_error_free(struct VirtLintError **err);
```

### Golang library API

The Golang API is written on top of C API and it too tries to mimic the Rust API closely.

```go
vl, _ := VirtLint.New(conn)
defer vl.Close()

err := vl.Validate(xml, tags, false)
if err != nil {
	return err
}

warn, err := vl.GetWarnings()
if err != nil {
	return err
}

for i := 0; i < len(warn); i++ {
	w := warn[i]
	fmt.Printf("Warning: tags=%v\tdomain=%v\tlevel=%v\tmsg=%s\n", w.Tags, w.Domain, w.Level, w.Msg)
}

```

### Rust `virt-lint` CLI

There is a small demo program under `tools/` that demonstrates capabilities of
the library. Its use is straightforward:

```shell
virt-lint -c qemu:///system -p /path/to/domain.xml
Warning: tags=["TAG_1", "TAG_2"]    domain=Domain  level=Error     msg=Domain would not fit into any host NUMA node
Warning: tags=["TAG_2"]             domain=Domain  level=Error     msg=Not enough free memory on any NUMA node
Warning: tags=["TAG_1", "TAG_3"]    domain=Node    level=Warning   msg=No suitable emulator
Warning: tags=["TAG_4"]             domain=Domain  level=Notice    msg=No free PCIe root ports found, hotplug might be not possible
```

As demo, similar binaries are written for C and Golang.

## Packaging

There is an ebuild that packages both Rust CLI binary and C library among with
pkg-config and header files. For building an RPM there also spec file, but
using `make rpm` is preferred as it creates source archives and other
preparation work necessary.

## Further development

There are plenty of areas to improve on:
- [ ] Replace tags ["TAG_1", "TAG_2", ...] with actual useful strings (e.g.
      project names like ["KubeVirt", "OpenStack", ...]).
- [ ] Write actually useful validators.
  - [ ] Write more of them.
- [ ] Separate validators from the library (for instance like [libosinfo] does
      it). These could be then updated independently from the library code.
  - [x] Utilize [mlua] crate and rewrite validators in Lua.
- [ ] Write bindings to other languages (Python, Perl?, Java?, JavaScript?)
- [ ] Don't link with libvirt.so in offline mode

[libosinfo]: https://libosinfo.org/
[mlua]: https://github.com/khvzak/mlua

## Contributing

Pull requests are welcome. Please make sure to update tests as appropriate.

## License

[LGPL-3.0](https://www.gnu.org/licenses/lgpl-3.0.html)

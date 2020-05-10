# Target support

Rust-LV2 uses pregenerated LV2 API bindings for different targets in order to increase usability and building speed. Rust has a lot of [supported targets](https://forge.rust-lang.org/release/platform-support.html), but our maintaining power is limited and therefore, only certain targets can be supported.

A target is supported by Rust-LV2 if a binding was generated for it. This however requires that there is a [maintainer](https://github.com/orgs/RustAudio/teams/lv2-maintainers) who has access to a machine that runs this target and who can generate and verify bindings on this machine. The bindings itself are generated with the [LV2 systool](sys/tool/) and verified by building the [example plugins of the book](docs) and testing them with a host of that target.

There are some targets that have a binding and a maintainer, but that haven't been verified yet. These targets have only experimental support and are gated behind optional crate features.

## Supported targets

| Target | Maintainer | Status | Last Verification |
|--------|------------|--------|-------------------|
| `x86_64-unknown-linux-gnu` | @Janonard | Supported | 10. of May 2020, using [Carla](https://github.com/falkTX/Carla) v2.1 |
| `x86-unknown-linux-gnu` | @Janonard | Supported | TODO |
| `x86_64-pc-windows-msvc` | @Janonard | Experimental | TODO |
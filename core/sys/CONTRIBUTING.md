## Updating LV2 bindings

Since LV2 is a standard header and not a library header, it is bundled inside this crate's source code.

The `bindings.rs` file is generated ahead-of-time using [bindgen](https://github.com/rust-lang/rust-bindgen), and the following command: 

```bash
bindgen \
    --no-derive-debug \
    --whitelist-type "LV2.*" \
    --whitelist-var "LV2_.*" \
    --generate-inline-functions \
    src/lv2.h -o src/bindings.rs
```

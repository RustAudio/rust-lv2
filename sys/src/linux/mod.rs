#[cfg_attr(target_arch = "x86", path = "x86.rs")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64.rs")]
//bindings are identical between soft float arm, hard float arm and aarch64
#[cfg_attr(
    all(
        target_arch = "arm",
        target_arch = "aarch64",
        feature = "experimental-targets"
    ),
    path = "arm.rs"
)]
mod toto;
pub use toto::*;

#[cfg_attr(target_arch = "x86", path = "x86.rs")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64.rs")]
#[cfg_attr(
    not(any(target_arch = "x84", target_arch = "x86_64")),
    path = "unsupported.rs"
)]
mod bindings;
pub use bindings::*;

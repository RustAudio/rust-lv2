#[cfg_attr(target_arch = "x86", path = "x86.rs")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64.rs")]
#[cfg_attr(target_arch = "arm", path = "arm.rs")]
#[cfg_attr(target_arch = "aarch64", path = "aarch64.rs")]
mod unsupported;
pub use unsupported::*;

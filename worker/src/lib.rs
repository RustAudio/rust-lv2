use lv2_core::plugin::Plugin;

/// Trait to provide worker extension to LV2 plugins.
///
/// The worker extension allows plugins to schedule work that must be performed in another thread. Plugins can use this interface to safely perform work that is not real-time safe, and receive the result in the run context. The details of threading are managed by the host, allowing plugins to be simple and portable while using resources more efficiently.
pub trait Worker: Plugin {
    /// The work to do in a non-real-time thread
    fn work();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

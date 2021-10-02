use urid::URID;

/// The subject of an Option, i.e. what the option applies to.
///
/// For instance, using a given Option `Foo`:
/// * A value of [`Subject::Instance`] means we are referring to the instance's `Foo` option;
/// * A value of [`Subject::Port`] means we are referring to a given port's `Foo` option;
/// * … and so on.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Subject {
    /// The option applies to the instance itself.
    Instance,
    /// The option applies to some specific named ([`URID`](urid::URID)) resource.
    /// The inner subject is the URID of said resource.
    Resource(URID),
    /// The option applies to some blank node.
    /// The inner value is a blank node identifier, which is valid only within the current local scope.
    Blank(u32),
    /// This option applies to a port on the instance.
    /// The inner value is the port's index.
    Port(u32), // TODO: handle PortIndex more gracefully
}

impl Subject {
    #[inline]
    pub(crate) fn from_raw(
        context: lv2_sys::LV2_Options_Context,
        subject: u32,
    ) -> core::option::Option<Self> {
        match context {
            lv2_sys::LV2_Options_Context_LV2_OPTIONS_INSTANCE => Some(Subject::Instance),
            lv2_sys::LV2_Options_Context_LV2_OPTIONS_RESOURCE => {
                Some(Subject::Resource(URID::new(subject)?))
            }
            lv2_sys::LV2_Options_Context_LV2_OPTIONS_BLANK => Some(Subject::Blank(subject)),
            lv2_sys::LV2_Options_Context_LV2_OPTIONS_PORT => Some(Subject::Port(subject)),
            _ => None,
        }
    }
}

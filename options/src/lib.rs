//! This crate enables the use of the [Options](http://lv2plug.in/ns/ext/options) LV2 Extension.
//!
//! Options are configuration values which the host passes to the plugin (or its UI) at runtime.
//!
//! There are two facilities available for plugins to deal with options:
//!
//! * The [`OptionsList`](features::OptionsList) [feature](lv2_core::feature), to get a list of all
//!   available options at directly instantiation time.
//! * The [`OptionsInterface`](extensions::OptionsInterface) [extension interface](lv2_core::extension),
//!   to allow the host to dynamically set or retrieve options after instantiation.
//!
//! Note that this extension is only for allowing hosts to configure plugins, and is not a "live"
//! control mechanism.
//! For real-time control, use event-based control via an [`AtomPort`](lv2_atom::port::AtomPort)
//! with a [`Sequence`](lv2_atom::sequence::Sequence) buffer.
//!
//! See the [LV2 Options documentation](http://lv2plug.in/ns/ext/options) for more information.

pub use option::error::OptionsError;
pub use option::request;
pub use option::subject::Subject;
pub use option::value::OptionValue;
pub use option::OptionType;

pub mod collection;
pub mod extensions;
pub mod list;
mod option;

/// Contains the [`OptionsList`](features::OptionsList) feature.
pub mod features {
    pub use crate::list::OptionsList;
}

/// Prelude of `lv2_options` for wildcard usage.
pub mod prelude {
    pub use crate::extensions::{OptionsDescriptor, OptionsInterface};
    pub use crate::list::OptionsList;
    pub use crate::OptionsError;
    pub use crate::Subject;
}

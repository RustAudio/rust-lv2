use lv2_atom::Atom;
use urid::UriBound;

pub mod error;
pub mod subject;
pub mod value;
pub mod request;

/// A trait representing an LV2 Option type.
///
/// # Example
///
/// This example implements a simple option type named "MyIntOption" backed by an Int atom.
///
/// ```
/// use lv2_options::OptionType;
/// use urid::*;
///
/// #[uri("urn:lv2_options:test:SomeIntOption")]
/// pub struct SomeIntOption(i32);
///
/// impl OptionType for SomeIntOption {
///     type AtomType = lv2_atom::scalar::Int;
///
///     fn from_option_value(value: &i32) -> Option<Self> {
///         Some(Self(*value))
///     }
///
///     fn as_option_value(&self) -> &i32 {
///         &self.0
///     }
/// }
/// ```
pub trait OptionType: UriBound + Sized {
    type AtomType: UriBound;

    /// Creates a new instance of this Option type from a given atom value.
    ///
    /// This method may return `None` if the Atom's value is invalid for this option type.
    ///
    /// This method is used to store option data when received by the host.
    fn from_option_value<'a>(value: <Self::AtomType as Atom<'a, 'a>>::ReadHandle) -> Option<Self> where Self::AtomType: Atom<'a, 'a>;

    /// Returns this Option's value as a reference to its Atom type.
    ///
    /// This method is used to send the option's value to the host when it is requested.
    fn as_option_value<'a>(&'a self) -> <Self::AtomType as Atom<'a, 'a>>::ReadHandle  where Self::AtomType: Atom<'a, 'a>;
}

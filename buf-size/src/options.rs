use lv2_atom::Atom;
use urid::UriBound;

/// A simple macro to automate the definition of the u32 options available in this module
macro_rules! make_option {
    ($name:ident, $uri:expr) => {
        pub struct $name(i32);

        impl $name {
            #[inline]
            pub fn get(&self) -> u32 {
                self.0 as u32
            }
        }

        unsafe impl UriBound for $name {
            const URI: &'static [u8] = $uri;
        }

        impl lv2_options::OptionType for $name {
            type AtomType = lv2_atom::scalar::Int;

            #[inline]
            fn from_option_value<'a>(
                value: <lv2_atom::scalar::Int as Atom<'a, 'a>>::ReadHandle,
            ) -> Option<Self> {
                Some(Self((*value)))
            }

            #[inline]
            fn as_option_value<'a>(&'a self) -> <lv2_atom::scalar::Int as Atom<'a, 'a>>::ReadHandle {
                &self.0
            }
        }
    };
}

make_option!(MinBlockLength, lv2_sys::LV2_BUF_SIZE__minBlockLength);
make_option!(MaxBlockLength, lv2_sys::LV2_BUF_SIZE__maxBlockLength);
make_option!(
    NominalBlockLength,
    lv2_sys::LV2_BUF_SIZE__nominalBlockLength
);
make_option!(SequenceSize, lv2_sys::LV2_BUF_SIZE__sequenceSize);

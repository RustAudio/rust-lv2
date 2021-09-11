use urid::{URIDCollection, Map};
use crate::request::OptionRequestList;
use crate::{OptionsError, OptionValue};
use crate::list::OptionsList;
use crate::option::request::OptionRequest;

pub trait OptionsCollection: Sized {
    type Serializer;

    #[inline]
    fn new_serializer<'a, M: Map + ?Sized>(map: &M) -> Option<OptionsSerializer<Self>>
        where Self::Serializer: OptionsSerializationContext<'a, Self> { // FIXME
        Some(OptionsSerializer { inner: Self::Serializer::from_map(map)? })
    }
}

#[doc(hidden)]
pub mod implementation {
    use crate::{OptionType, OptionsError, OptionValue};
    use std::marker::PhantomData;
    use urid::{URID, URIDCollection, Map};
    use crate::collection::{OptionsSerializationContext, OptionsCollection};
    use crate::option::request::OptionRequest;
    use lv2_atom::{Atom, BackAsSpace};
    use lv2_atom::scalar::ScalarAtom;

    pub struct OptionTypeSerializationContext<O: OptionType> {
        option_urid: URID<O>,
        option_type_atom_urid: URID<O::AtomType>
    }

    impl<'a, O: OptionType> OptionsCollection for O
        where <O as OptionType>::AtomType: BackAsSpace<'a>,
              <<O as OptionType>::AtomType as Atom<'a, 'a>>::ReadParameter: Default {
        type Serializer = OptionTypeSerializationContext<O>;
    }

    impl<O: OptionType> URIDCollection for OptionTypeSerializationContext<O> {
        #[inline]
        fn from_map<M: Map + ?Sized>(map: &M) -> Option<Self> {
            Some(Self {
                option_urid: map.populate_collection()?,
                option_type_atom_urid: map.populate_collection()?,
            })
        }
    }

    impl<'a, O: OptionType> OptionsSerializationContext<'a, O> for OptionTypeSerializationContext<O>
        where <O as OptionType>::AtomType: BackAsSpace<'a>,
              <<O as OptionType>::AtomType as Atom<'a, 'a>>::ReadParameter: Default {
        #[inline]
        fn deserialize_new(&self, option: &'a OptionValue) -> Option<O> {
            option.read(self.option_urid, self.option_type_atom_urid, Default::default())
        }

        fn deserialize_to(&self, options: &mut O, option: &OptionValue) -> Result<(), OptionsError> {
            todo!()
        }

        fn respond_to_request<'r>(&self, options: &'r O, requests: &'r mut OptionRequest) -> Result<(), OptionsError> {
            todo!()
        }
    }
}

pub struct OptionsSerializer<T: OptionsCollection> {
    inner: T::Serializer
}

impl<T: OptionsCollection> OptionsSerializer<T> {
    pub fn deserialize_new(&self, list: &OptionsList) -> Option<T> {
        todo!()
    }

    pub fn deserialize_to(&self, options: &mut T, list: &OptionsList) -> Result<(), OptionsError> {
        todo!()
    }

    pub fn respond_to_requests<'a>(&self, options: &T, requests: &mut OptionRequestList) -> Result<(), OptionsError> {
        todo!()
    }
}

pub trait OptionsSerializationContext<'a, T: OptionsCollection>: URIDCollection {
    fn deserialize_new(&self, option: &'a OptionValue) -> Option<T>;

    fn deserialize_to(&self, options: &mut T, option: &OptionValue) -> Result<(), OptionsError>;

    fn respond_to_request<'r>(&self, options: &'r T, request: &'r mut OptionRequest) -> Result<(), OptionsError>;
}
use urid::{Map, URIDCollection};

use crate::list::OptionsList;
use crate::option::request::OptionRequest;
use crate::request::OptionRequestList;
use crate::OptionsError;

mod option;
mod option_value;

pub use lv2_options_derive::OptionsCollection;

pub trait OptionsCollection: Sized {
    type Serializer: OptionsSerializationContext<Self>;
}

pub struct Options<T: OptionsCollection + Sized> {
    pub values: T,
    serializer: T::Serializer,
}

impl<T: OptionsCollection> Options<T> {
    #[inline]
    pub fn default<M: Map + ?Sized>(features_map: &M) -> Option<Self>
    where
        T: Default,
    {
        Some(Self {
            serializer: T::Serializer::from_map(features_map)?,
            values: Default::default(),
        })
    }

    #[inline]
    pub fn deserialize_new<'a, M: Map + ?Sized>(
        features_map: &M,
        options: &'a OptionsList<'a>,
    ) -> Result<Self, OptionsError> {
        let serializer = T::Serializer::from_map(features_map).ok_or(OptionsError::Unknown)?;
        let values = serializer.deserialize_new(options)?;

        Ok(Self { serializer, values })
    }

    #[inline]
    pub fn deserialize(&mut self, list: &OptionsList) -> Result<(), OptionsError> {
        self.serializer.deserialize_to(&mut self.values, list)
    }

    pub fn respond_to_requests<'a>(
        &'a self,
        requests: &mut OptionRequestList<'a>,
    ) -> Result<(), OptionsError> {
        for request in requests {
            self.serializer.respond_to_request(&self.values, request)?
        }

        Ok(())
    }
}

pub trait OptionsSerializationContext<T: OptionsCollection>: URIDCollection {
    fn deserialize_new(&self, options: &OptionsList) -> Result<T, OptionsError>;

    fn deserialize_to(
        &self,
        destination: &mut T,
        options: &OptionsList,
    ) -> Result<(), OptionsError>;

    fn respond_to_request<'a>(
        &self,
        options: &'a T,
        requests: &mut OptionRequest<'a>,
    ) -> Result<(), OptionsError>;
}

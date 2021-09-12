use crate::list::OptionsList;
use crate::request::OptionRequestList;
use crate::OptionsError;
use urid::{Map, URIDCollection};

pub trait OptionsCollection: Sized {
    type Serializer: OptionsSerializationContext<Self>;

    #[inline]
    fn new_serializer<'a, M: Map + ?Sized>(map: &M) -> Option<OptionsSerializer<Self>> {
        // FIXME
        Some(OptionsSerializer {
            inner: Self::Serializer::from_map(map)?,
        })
    }
}

pub struct OptionsSerializer<T: OptionsCollection> {
    inner: T::Serializer,
}

impl<T: OptionsCollection> OptionsSerializer<T> {
    #[inline]
    pub fn deserialize_new<'a>(&'a self, list: &'a OptionsList<'a>) -> Result<T, OptionsError> {
        self.inner.deserialize_new(list)
    }

    #[inline]
    pub fn deserialize_to(&self, options: &mut T, list: &OptionsList) -> Result<(), OptionsError> {
        self.inner.deserialize_to(options, list)
    }

    #[inline]
    pub fn respond_to_requests<'a>(
        &self,
        options: &'a T,
        requests: &mut OptionRequestList<'a>,
    ) -> Result<(), OptionsError> {
        self.inner.respond_to_requests(options, requests)
    }
}

pub trait OptionsSerializationContext<T: OptionsCollection>: URIDCollection {
    fn deserialize_new(&self, options: &OptionsList) -> Result<T, OptionsError>;

    fn deserialize_to(
        &self,
        destination: &mut T,
        options: &OptionsList,
    ) -> Result<(), OptionsError>;

    fn respond_to_requests<'a>(
        &self,
        options: &'a T,
        requests: &mut OptionRequestList<'a>,
    ) -> Result<(), OptionsError>;
}

#[doc(hidden)]
pub mod __implementation {
    use crate::collection::{OptionsCollection, OptionsSerializationContext};
    use crate::list::OptionsList;
    use crate::request::OptionRequestList;
    use crate::{OptionType, OptionsError};
    use urid::{Map, URIDCollection, URID};

    pub mod option_value {
        use super::*;
        pub struct OptionTypeSerializationContext<O: OptionType> {
            option_urid: URID<O>,
            option_type_atom_urid: URID<O::AtomType>,
        }

        impl<'a, O: OptionType> OptionsCollection for O {
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

        impl<O: OptionType> OptionsSerializationContext<O> for OptionTypeSerializationContext<O> {
            fn deserialize_new(&self, options: &OptionsList) -> Result<O, OptionsError> {
                for option in options {
                    match option.read(self.option_urid, self.option_type_atom_urid) {
                        Ok(value) => return Ok(value),
                        Err(OptionsError::BadKey) => {}
                        Err(e) => return Err(e),
                    }
                }

                Err(OptionsError::BadKey)
            }

            fn deserialize_to(
                &self,
                destination: &mut O,
                options: &OptionsList,
            ) -> Result<(), OptionsError> {
                for option in options {
                    match option.read(self.option_urid, self.option_type_atom_urid) {
                        Ok(value) => {
                            *destination = value;
                            return Ok(());
                        }
                        Err(OptionsError::BadKey) => {}
                        Err(e) => return Err(e),
                    }
                }

                Err(OptionsError::BadKey)
            }

            fn respond_to_requests<'a>(
                &self,
                options: &'a O,
                requests: &mut OptionRequestList<'a>,
            ) -> Result<(), OptionsError> {
                for request in requests {
                    match request.try_respond(self.option_urid, self.option_type_atom_urid, options)
                    {
                        Ok(()) => return Ok(()),
                        Err(OptionsError::BadKey) => {}
                        Err(e) => return Err(e),
                    }
                }

                Ok(())
            }
        }
    }

    mod option {
        use super::*;

        pub struct OptionSerializationContext<O: OptionsCollection> {
            inner: O::Serializer,
        }

        impl<'a, O: OptionsCollection> OptionsCollection for Option<O> {
            type Serializer = OptionSerializationContext<O>;
        }

        impl<O: OptionsCollection> URIDCollection for OptionSerializationContext<O> {
            #[inline]
            fn from_map<M: Map + ?Sized>(map: &M) -> Option<Self> {
                Some(Self {
                    inner: map.populate_collection()?,
                })
            }
        }

        impl<O: OptionsCollection> OptionsSerializationContext<Option<O>>
            for OptionSerializationContext<O>
        {
            fn deserialize_new(&self, options: &OptionsList) -> Result<Option<O>, OptionsError> {
                match self.inner.deserialize_new(options) {
                    Ok(value) => Ok(Some(value)),
                    Err(OptionsError::BadKey) => Ok(None),
                    Err(e) => Err(e),
                }
            }

            fn deserialize_to(
                &self,
                destination: &mut Option<O>,
                options: &OptionsList,
            ) -> Result<(), OptionsError> {
                let result = match destination {
                    Some(value) => self.inner.deserialize_to(value, options),
                    None => match self.inner.deserialize_new(options) {
                        Ok(v) => {
                            destination.insert(v);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                };

                match result {
                    Ok(()) | Err(OptionsError::BadKey) => Ok(()),
                    Err(e) => Err(e),
                }
            }

            fn respond_to_requests<'a>(
                &self,
                options: &'a Option<O>,
                requests: &mut OptionRequestList<'a>,
            ) -> Result<(), OptionsError> {
                if let Some(value) = options {
                    self.inner.respond_to_requests(value, requests)
                } else {
                    Ok(())
                }
            }
        }
    }
}

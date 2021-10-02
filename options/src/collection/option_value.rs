use crate::collection::{OptionsCollection, OptionsSerializationContext};
use crate::list::OptionsList;
use crate::option::request::OptionRequest;
use crate::{OptionType, OptionsError};
use urid::{Map, URIDCollection, URID};

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

    fn respond_to_request<'a>(
        &self,
        options: &'a O,
        request: &mut OptionRequest<'a>,
    ) -> Result<(), OptionsError> {
        request.try_respond(self.option_urid, self.option_type_atom_urid, options)
    }
}

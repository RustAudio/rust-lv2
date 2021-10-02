use crate::option::request::OptionRequest;

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
                    let _ = destination.insert(v);
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

    fn respond_to_request<'a>(
        &self,
        options: &'a Option<O>,
        requests: &mut OptionRequest<'a>,
    ) -> Result<(), OptionsError> {
        if let Some(value) = options {
            self.inner.respond_to_request(value, requests)
        } else {
            Ok(())
        }
    }
}

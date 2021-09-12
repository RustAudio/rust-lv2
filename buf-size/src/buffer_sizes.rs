use crate::options::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct BufferSizes {
    min_block_length: Option<MinBlockLength>,
    max_block_length: Option<MaxBlockLength>,
    nominal_block_length: Option<NominalBlockLength>,
    sequence_size: Option<SequenceSize>,
}

const _: () = {
    extern crate lv2_options as _lv2_options;
    extern crate urid as _urid;
    use _lv2_options::collection::{OptionsCollection, OptionsSerializationContext};
    use _lv2_options::features::OptionsList;
    use _lv2_options::request::OptionRequestList;
    use _lv2_options::OptionsError;

    use _urid::*;

    #[derive(_urid::URIDCollection)]
    pub struct BufferSizesSerializationContext {
        pub min_block_length: <Option<MinBlockLength> as OptionsCollection>::Serializer,
        pub max_block_length: <Option<MaxBlockLength> as OptionsCollection>::Serializer,
        pub nominal_block_length: <Option<NominalBlockLength> as OptionsCollection>::Serializer,
        pub sequence_size: <Option<SequenceSize> as OptionsCollection>::Serializer,
    }

    impl OptionsCollection for BufferSizes {
        type Serializer = BufferSizesSerializationContext;
    }

    impl OptionsSerializationContext<BufferSizes> for BufferSizesSerializationContext {
        fn deserialize_new(&self, options: &OptionsList) -> Result<BufferSizes, OptionsError> {
            Ok(BufferSizes {
                min_block_length: self.min_block_length.deserialize_new(options)?,
                max_block_length: self.max_block_length.deserialize_new(options)?,
                nominal_block_length: self.nominal_block_length.deserialize_new(options)?,
                sequence_size: self.sequence_size.deserialize_new(options)?,
            })
        }

        fn deserialize_to(
            &self,
            destination: &mut BufferSizes,
            options: &OptionsList,
        ) -> Result<(), OptionsError> {
            self.min_block_length
                .deserialize_to(&mut destination.min_block_length, options)?;
            self.max_block_length
                .deserialize_to(&mut destination.max_block_length, options)?;
            self.nominal_block_length
                .deserialize_to(&mut destination.nominal_block_length, options)?;
            self.sequence_size
                .deserialize_to(&mut destination.sequence_size, options)?;

            Ok(())
        }

        fn respond_to_requests<'a>(
            &self,
            options: &'a BufferSizes,
            requests: &mut OptionRequestList<'a>,
        ) -> Result<(), OptionsError> {
            self.min_block_length
                .respond_to_requests(&options.min_block_length, requests)?;
            self.max_block_length
                .respond_to_requests(&options.max_block_length, requests)?;
            self.nominal_block_length
                .respond_to_requests(&options.nominal_block_length, requests)?;
            self.sequence_size
                .respond_to_requests(&options.sequence_size, requests)?;

            Ok(())
        }
    }
};

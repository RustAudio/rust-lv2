use crate::options::*;
use lv2_atom::prelude::Int;
use lv2_options::features::OptionsList;
use urid::{Map, URIDCollection, URID};

#[derive(URIDCollection)]
pub struct BufferSizesURIDCollection {
    pub atom_int: URID<Int>,
    pub min_block_length: URID<MinBlockLength>,
    pub max_block_length: URID<MaxBlockLength>,
    pub nominal_block_length: URID<NominalBlockLength>,
    pub sequence_size: URID<SequenceSize>,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BufferSizes {
    min_block_length: Option<u32>,
    max_block_length: Option<u32>,
    nominal_block_length: Option<u32>,
    sequence_size: Option<u32>,
}

impl BufferSizes {
    pub fn from_options(options: &OptionsList, mapper: &impl Map) -> Self {
        let collection = match mapper.populate_collection() {
            Some(c) => c,
            None => return Default::default(),
        };
        BufferSizes::from_options_with_urids(options, &collection)
    }

    pub fn from_options_with_urids(
        options: &OptionsList,
        urids: &BufferSizesURIDCollection,
    ) -> Self {
        BufferSizes {
            min_block_length: options
                .read(urids.min_block_length, urids.atom_int, ())
                .map(|x| x.get()),
            max_block_length: options
                .read(urids.max_block_length, urids.atom_int, ())
                .map(|x| x.get()),
            nominal_block_length: options
                .read(urids.nominal_block_length, urids.atom_int, ())
                .map(|x| x.get()),
            sequence_size: options
                .read(urids.sequence_size, urids.atom_int, ())
                .map(|x| x.get()),
        }
    }
}

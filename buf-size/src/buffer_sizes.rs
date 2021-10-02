use crate::options::*;
use lv2_options::prelude::OptionsCollection;

#[derive(Copy, Clone, Debug, Default, OptionsCollection)]
pub struct BufferSizes {
    min_block_length: Option<MinBlockLength>,
    max_block_length: Option<MaxBlockLength>,
    nominal_block_length: Option<NominalBlockLength>,
    sequence_size: Option<SequenceSize>,
}

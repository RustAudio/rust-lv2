pub mod chunk;
pub mod object;
pub mod scalar;
pub mod sequence;
pub mod string;
pub mod tuple;
pub mod vector;

use urid::*;

/// An URID collection of all standard atom types, provided for convenience.
#[derive(Clone)]
pub struct AtomURIDCollection {
    pub blank: URID<object::Blank>,
    pub double: URID<scalar::Double>,
    pub float: URID<scalar::Float>,
    pub int: URID<scalar::Int>,
    pub long: URID<scalar::Long>,
    pub urid: URID<scalar::AtomURID>,
    pub bool: URID<scalar::Bool>,
    pub vector: URID<vector::Vector>,
    pub chunk: URID<chunk::Chunk>,
    pub literal: URID<string::Literal>,
    pub object: URID<object::Object>,
    pub property: URID<object::Property>,
    pub string: URID<string::String>,
    pub tuple: URID<tuple::Tuple>,
    pub sequence: URID<sequence::Sequence>,
}

impl URIDCollection for AtomURIDCollection {
    fn from_map<M: Map + ?Sized>(map: &M) -> Option<Self> {
        Some(Self {
            blank: map.map_type()?,
            double: map.map_type()?,
            float: map.map_type()?,
            int: map.map_type()?,
            long: map.map_type()?,
            urid: map.map_type()?,
            bool: map.map_type()?,
            vector: map.map_type()?,
            chunk: map.map_type()?,
            literal: map.map_type()?,
            object: map.map_type()?,
            property: map.map_type()?,
            string: map.map_type()?,
            tuple: map.map_type()?,
            sequence: map.map_type()?,
        })
    }
}

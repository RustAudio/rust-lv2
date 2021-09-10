pub mod chunk;
pub mod object;
pub mod scalar;
pub mod sequence;
pub mod string;
pub mod tuple;
pub mod vector;

pub use crate::header::AtomHeader;
use urid::*;

#[derive(Clone, URIDCollection)]
/// Collection with the URIDs of all `UriBound`s in this crate.
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

//! Since this crate uses `bindgen` to create the C API bindings, you need to have clang installed on your machine.
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(clippy::all)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::{
    LV2_ATOM__Atom, LV2_ATOM__AtomPort, LV2_ATOM__Blank, LV2_ATOM__Bool, LV2_ATOM__Chunk,
    LV2_ATOM__Double, LV2_ATOM__Event, LV2_ATOM__Float, LV2_ATOM__Int, LV2_ATOM__Literal,
    LV2_ATOM__Long, LV2_ATOM__Number, LV2_ATOM__Object, LV2_ATOM__Path, LV2_ATOM__Property,
    LV2_ATOM__Resource, LV2_ATOM__Sequence, LV2_ATOM__Sound, LV2_ATOM__String, LV2_ATOM__Tuple,
    LV2_ATOM__Vector, LV2_ATOM__atomTransfer, LV2_ATOM__beatTime, LV2_ATOM__bufferType,
    LV2_ATOM__childType, LV2_ATOM__eventTransfer, LV2_ATOM__frameTime, LV2_ATOM__supports,
    LV2_ATOM__timeUnit, LV2_Atom, LV2_Atom_Bool, LV2_Atom_Double, LV2_Atom_Event,
    LV2_Atom_Event__bindgen_ty_1 as LV2_Atom_Event_Timestamp, LV2_Atom_Float, LV2_Atom_Int,
    LV2_Atom_Literal, LV2_Atom_Literal_Body, LV2_Atom_Long, LV2_Atom_Object, LV2_Atom_Object_Body,
    LV2_Atom_Property, LV2_Atom_Property_Body, LV2_Atom_Sequence, LV2_Atom_Sequence_Body,
    LV2_Atom_String, LV2_Atom_Tuple, LV2_Atom_URID, LV2_Atom_Vector, LV2_Atom_Vector_Body,
    LV2_ATOM_PREFIX, LV2_ATOM_REFERENCE_TYPE, LV2_ATOM_URI, LV2_ATOM__URI, LV2_ATOM__URID,
};

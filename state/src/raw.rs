use crate::StateErr;
use atom::prelude::*;
use atom::space::*;
use atom::AtomHandle;
use atom::AtomHeader;
use std::collections::HashMap;
use std::ffi::c_void;
use std::marker::PhantomData;
use urid::*;

/// Property storage handle.
///
/// This handle can be used to store the properties of a plugin. It uses the atom system to encode the properties and is backed by a storage callback function.
///
/// The written properties a buffered and flushed when requested. Create new properties by calling [`draft`](#method.draft) and write them like any other atom. Once you are done, you can commit your properties by calling [`commit_all`](#method.commit_all) or [`commit`](#method.commit). You have to commit manually: Uncommitted properties will be discarded when the handle is dropped.
pub struct StoreHandle<'a> {
    properties: HashMap<URID, AlignedVec<AtomHeader>>,
    store_fn: sys::LV2_State_Store_Function,
    handle: sys::LV2_State_Handle,
    lifetime: PhantomData<&'a mut c_void>,
}

impl<'a> StoreHandle<'a> {
    /// Create a new store handle.
    pub fn new(store_fn: sys::LV2_State_Store_Function, handle: sys::LV2_State_Handle) -> Self {
        StoreHandle {
            properties: HashMap::new(),
            store_fn,
            handle,
            lifetime: PhantomData,
        }
    }

    /// Draft a new property.
    ///
    /// This will return a new handle to create a property. Once the property is completely written, you can commit it by calling [`commit`](#method.commit) or [`commit_all`](#method.commit_all). Then, and only then, it will be saved by the host.
    ///
    /// If you began to write a property and don't want the written things to be stored, you can discard it with [`discard`](#method.discard) or [`discard_all`](#method.discard_all).
    pub fn draft<K: ?Sized>(&mut self, property_key: URID<K>) -> StatePropertyWriter {
        let property_key = property_key.into_general();
        let space = self
            .properties
            .entry(property_key.into_general())
            .or_insert_with(AlignedVec::new);

        StatePropertyWriter::new(space.write())
    }

    /// Internal helper function to store a property.
    fn commit_pair<K: ?Sized>(
        store_fn: sys::LV2_State_Store_Function,
        handle: sys::LV2_State_Handle,
        key: URID<K>,
        space: AlignedVec<AtomHeader>,
    ) -> Result<(), StateErr> {
        let store_fn = store_fn.ok_or(StateErr::BadCallback)?;
        let space = space.as_space();
        let atom = unsafe { space.read().next_atom() }.map_err(|_| StateErr::BadData)?;

        let key = key.get();
        let data_ptr = atom.body().as_bytes().as_ptr() as *const c_void;
        let data_size = atom.header().size_of_body();
        let data_type = atom.header().urid().get();
        let flags: u32 = (sys::LV2_State_Flags::LV2_STATE_IS_POD
            | sys::LV2_State_Flags::LV2_STATE_IS_PORTABLE)
            .into();
        StateErr::from(unsafe { (store_fn)(handle, key, data_ptr, data_size, data_type, flags) })
    }

    /// Commit all created properties.
    ///
    /// This will also clear the property buffer.
    pub fn commit_all(&mut self) -> Result<(), StateErr> {
        for (key, space) in self.properties.drain() {
            Self::commit_pair(self.store_fn, self.handle, key, space)?;
        }
        Ok(())
    }

    /// Commit one specific property.
    ///
    /// This method returns `None` if the requested property was not marked for commit, `Some(Ok(()))` if the property was stored and `Some(Err(_))` if an error occurred while storing the property.
    pub fn commit<K: ?Sized>(&mut self, key: URID<K>) -> Option<Result<(), StateErr>> {
        let key = key.into_general();
        let space = self.properties.remove(&key)?;
        Some(Self::commit_pair(self.store_fn, self.handle, key, space))
    }

    /// Discard all drafted properties.
    pub fn discard_all(&mut self) {
        self.properties.clear();
    }

    /// Discard a drafted property.
    ///
    /// If no property with the given key was drafted before, this is a no-op.
    pub fn discard<K: ?Sized>(&mut self, key: URID<K>) {
        self.properties.remove(&key.into_general());
    }
}

/// Writing handle for properties.
pub struct StatePropertyWriter<'a> {
    cursor: AlignedVecCursor<'a, AtomHeader>,
    initialized: bool,
}

impl<'a> StatePropertyWriter<'a> {
    /// Create a new property writer that uses the given space head.
    pub fn new(cursor: AlignedVecCursor<'a, AtomHeader>) -> Self {
        Self {
            cursor,
            initialized: false,
        }
    }

    /// Initialize the property.
    ///
    /// This works like any other atom writer: You have to provide the URID of the atom type you want to write, as well as the type-specific parameter. If the property hasn't been initialized before, it will be initialized and the writing handle is returned. Otherwise, `Err(StateErr::Unknown)` is returned.
    pub fn init<A: Atom>(
        &'a mut self,
        urid: URID<A>,
    ) -> Result<<A::WriteHandle as AtomHandle<'a>>::Handle, StateErr> {
        if !self.initialized {
            self.initialized = true;
            self.cursor.write_atom(urid).map_err(|_| StateErr::Unknown)
        } else {
            Err(StateErr::Unknown)
        }
    }
}

/// Property retrieval handle.
pub struct RetrieveHandle<'a> {
    retrieve_fn: sys::LV2_State_Retrieve_Function,
    handle: sys::LV2_State_Handle,
    lifetime: PhantomData<&'a mut c_void>,
}

impl<'a> RetrieveHandle<'a> {
    /// Create a new retrieval handle that uses the given callback function and handle.
    pub fn new(
        retrieve_fn: sys::LV2_State_Retrieve_Function,
        handle: sys::LV2_State_Handle,
    ) -> Self {
        RetrieveHandle {
            retrieve_fn,
            handle,
            lifetime: PhantomData,
        }
    }

    /// Try to retrieve a property from the host.
    ///
    /// This method calls the internal retrieve callback with the given URID. If there's no property with the given URID, `Err(StateErr::NoProperty)` is returned. Otherwise, a reading handle is returned that contains the type and the data of the property and can interpret it as an atom.
    pub fn retrieve<K: ?Sized>(&self, key: URID<K>) -> Result<StatePropertyReader, StateErr> {
        let mut size: usize = 0;
        let mut type_: u32 = 0;
        let property_ptr: *const std::ffi::c_void = unsafe {
            (self.retrieve_fn.ok_or(StateErr::BadCallback)?)(
                self.handle,
                key.get(),
                &mut size,
                &mut type_,
                std::ptr::null_mut(),
            )
        };

        let type_ = URID::new(type_).ok_or(StateErr::Unknown)?;
        let space = if !property_ptr.is_null() {
            unsafe { std::slice::from_raw_parts(property_ptr as *const u8, size) }
        } else {
            return Err(StateErr::NoProperty);
        };

        Ok(StatePropertyReader::new(
            type_,
            AlignedSpace::from_bytes(space).map_err(|_| StateErr::BadData)?,
        ))
    }
}

/// Reading handle for properties.
///
/// This handle contains the type and the data of a property retrieved from the [`RetrieveHandle`](struct.RetrieveHandle.html).
pub struct StatePropertyReader<'a> {
    type_: URID,
    body: &'a AtomSpace,
}

impl<'a> StatePropertyReader<'a> {
    /// Create a new reading handle with the given type and data.
    pub fn new<T: ?Sized>(type_: URID<T>, body: &'a AtomSpace) -> Self {
        Self {
            type_: type_.into_general(),
            body,
        }
    }

    /// Return the type of the property.
    pub fn type_(&self) -> URID {
        self.type_
    }

    /// Return the data of the property.
    pub fn body(&self) -> &AtomSpace {
        self.body
    }

    /// Try to interpret the property as an atom.
    ///
    /// This works like any atom reader: You pass the URID of the atom type as well as the type-specific argument, and if the desired type is the actual type of the data, a read handle is returned.
    ///
    /// If the desired and actual data types don't match, `Err(StateErr::BadType)` is returned.
    pub fn read<A: Atom>(
        &self,
        urid: URID<A>,
    ) -> Result<<A::ReadHandle as AtomHandle<'a>>::Handle, StateErr> {
        if urid == self.type_ {
            unsafe { A::read(self.body) }.map_err(|_| StateErr::Unknown)
        } else {
            Err(StateErr::BadType)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::raw::*;
    use crate::storage::Storage;
    use atom::space::AlignedSpace;

    fn store(storage: &mut Storage, urids: &AtomURIDCollection) {
        let mut store_handle = storage.store_handle();

        store_handle
            .draft(URID::new(1).unwrap())
            .init(urids.int)
            .unwrap()
            .set(17)
            .unwrap();
        store_handle
            .draft(URID::new(2).unwrap())
            .init(urids.float)
            .unwrap()
            .set(1.0)
            .unwrap();

        store_handle.commit(URID::new(1).unwrap()).unwrap().unwrap();

        let mut vector_writer = store_handle.draft(URID::new(3).unwrap());
        let mut vector_writer = vector_writer
            .init(urids.vector)
            .unwrap()
            .of_type(urids.int)
            .unwrap();
        vector_writer.append(&[1, 2, 3, 4]).unwrap();

        store_handle.commit_all().unwrap();

        store_handle
            .draft(URID::new(4).unwrap())
            .init(urids.int)
            .unwrap()
            .set(0)
            .unwrap();
    }

    fn retrieve(storage: &mut Storage, urids: &AtomURIDCollection) {
        let retrieve_handle = storage.retrieve_handle();

        assert_eq!(
            17,
            *retrieve_handle
                .retrieve(URID::new(1).unwrap())
                .unwrap()
                .read(urids.int)
                .unwrap()
        );
        assert_eq!(
            1.0f32.to_ne_bytes(),
            retrieve_handle
                .retrieve(URID::new(2).unwrap())
                .unwrap()
                .read(urids.float)
                .unwrap()
                .to_ne_bytes()
        );
        assert_eq!(
            [1, 2, 3, 4],
            retrieve_handle
                .retrieve(URID::new(3).unwrap())
                .unwrap()
                .read(urids.vector)
                .unwrap()
                .of_type(urids.int)
                .unwrap()
        );
        assert!(retrieve_handle.retrieve(URID::new(4).unwrap()).is_err());
    }

    #[test]
    fn test_storage() {
        let map = HashURIDMapper::new();
        let urids = AtomURIDCollection::from_map(&map).unwrap();

        let mut storage = Storage::default();

        store(&mut storage, &urids);

        for (key, (type_, value)) in storage.iter() {
            match key.get() {
                1 => {
                    assert_eq!(urids.int, *type_);
                    assert_eq!(17, unsafe { *(value.as_slice() as *const _ as *const i32) });
                }
                2 => {
                    assert_eq!(urids.float, *type_);
                    assert_eq!(
                        1.0f32.to_ne_bytes(),
                        unsafe { *(value.as_slice() as *const _ as *const f32) }.to_ne_bytes()
                    );
                }
                3 => {
                    assert_eq!(urids.vector, *type_);
                    let space = AlignedSpace::from_bytes(value.as_slice()).unwrap();
                    let data = unsafe { Vector::read(space) }
                        .unwrap()
                        .of_type(urids.int)
                        .unwrap();
                    assert_eq!([1, 2, 3, 4], data);
                }
                _ => panic!("Invalid key!"),
            }
        }

        retrieve(&mut storage, &urids);
    }
}

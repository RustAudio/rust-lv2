use crate::interface::*;
use crate::StateErr;
use atom::prelude::*;
use atom::space::*;
use std::collections::HashMap;
use std::ffi::c_void;
use urid::prelude::*;

/// A handle to abstract state storage.
///
/// This handle buffers the written properties and flushes them at once. Create new properties by calling [`draft`](#method.draft) and write them like any other atom. Once you are done, you can commit your properties by calling [`commit_all`](#method.commit_all) or [`commit`](#method.commit). You have to commit manually: Uncommitted properties will be discarded when the handle is dropped.
pub struct RawStoreHandle {
    properties: HashMap<URID, SpaceElement>,
    store_fn: sys::LV2_State_Store_Function,
    handle: sys::LV2_State_Handle,
}

impl RawStoreHandle {
    /// Create a new store handle.
    pub fn new(store_fn: sys::LV2_State_Store_Function, handle: sys::LV2_State_Handle) -> Self {
        RawStoreHandle {
            properties: HashMap::new(),
            store_fn,
            handle,
        }
    }

    /// Internal helper function to store one property.
    fn commit_pair(
        store_fn: sys::LV2_State_Store_Function,
        handle: sys::LV2_State_Handle,
        key: URID,
        space: SpaceElement,
    ) -> Result<(), StateErr> {
        let store_fn = store_fn.ok_or(StateErr::BadCallback)?;
        let space: Vec<u8> = space.to_vec();
        let space = Space::from_slice(space.as_ref());
        let (header, data) = space
            .split_type::<sys::LV2_Atom>()
            .ok_or(StateErr::BadData)?;
        let data = data
            .split_raw(header.size as usize)
            .map(|(data, _)| data)
            .ok_or(StateErr::BadData)?;

        let key = key.get();
        let data_ptr = data as *const _ as *const c_void;
        let data_size = header.size as usize;
        let data_type = header.type_;
        let flags =
            sys::LV2_State_Flags_LV2_STATE_IS_POD | sys::LV2_State_Flags_LV2_STATE_IS_PORTABLE;
        StateErr::from(unsafe { (store_fn)(handle, key, data_ptr, data_size, data_type, flags) })
    }
}

impl StoreHandle for RawStoreHandle {
    fn draft(&mut self, property_key: URID) -> StatePropertyWriter {
        self.properties
            .insert(property_key, SpaceElement::default());
        StatePropertyWriter::new(SpaceHead::new(
            self.properties.get_mut(&property_key).unwrap(),
        ))
    }

    fn commit_all(&mut self) -> Result<(), StateErr> {
        for (key, space) in self.properties.drain() {
            Self::commit_pair(self.store_fn, self.handle, key, space)?;
        }
        Ok(())
    }

    fn commit(&mut self, key: URID) -> Option<Result<(), StateErr>> {
        let space = self.properties.remove(&key)?;
        Some(Self::commit_pair(self.store_fn, self.handle, key, space))
    }

    fn discard_all(&mut self) {
        self.properties.clear();
    }

    fn discard(&mut self, key: URID) {
        self.properties.remove(&key);
    }
}

impl RawRetrieveHandle {
    pub fn new(
        retrieve_fn: sys::LV2_State_Retrieve_Function,
        handle: sys::LV2_State_Handle,
    ) -> Self {
        RawRetrieveHandle {
            retrieve_fn,
            handle,
        }
    }
}

pub struct RawRetrieveHandle {
    retrieve_fn: sys::LV2_State_Retrieve_Function,
    handle: sys::LV2_State_Handle,
}

impl RetrieveHandle for RawRetrieveHandle {
    fn retrieve(&self, key: URID) -> Option<StatePropertyReader> {
        let mut size: usize = 0;
        let mut type_: u32 = 0;
        let property_ptr: *const std::ffi::c_void = unsafe {
            (self.retrieve_fn?)(
                self.handle,
                key.get(),
                &mut size,
                &mut type_,
                std::ptr::null_mut(),
            )
        };

        let type_ = URID::new(type_)?;
        let space = if !property_ptr.is_null() {
            unsafe { std::slice::from_raw_parts(property_ptr as *const u8, size) }
        } else {
            return None;
        };

        Some(StatePropertyReader::new(type_, Space::from_slice(space)))
    }
}

#[cfg(test)]
mod tests {
    use crate::raw::*;
    use atom::space::Space;
    use std::collections::HashMap;
    use std::ffi::c_void;
    use urid::mapper::*;

    struct Storage {
        items: HashMap<URID, (URID, Vec<u8>)>,
    }

    impl Storage {
        fn new() -> Self {
            Self {
                items: HashMap::new(),
            }
        }

        fn store(&mut self, key: URID, type_: URID, value: &[u8]) {
            self.items.insert(key, (type_, value.to_owned()));
        }

        unsafe extern "C" fn extern_store(
            handle: sys::LV2_State_Handle,
            key: u32,
            value: *const c_void,
            size: usize,
            type_: u32,
            _: u32,
        ) -> sys::LV2_State_Status {
            let handle = (handle as *mut Self).as_mut().unwrap();
            let key = URID::new(key).unwrap();
            let value = std::slice::from_raw_parts(value as *const u8, size);
            let type_ = URID::new(type_).unwrap();
            handle.store(key, type_, value);
            sys::LV2_State_Status_LV2_STATE_SUCCESS
        }

        fn retrieve(&self, key: URID) -> Option<(URID, &[u8])> {
            self.items
                .get(&key)
                .map(|(urid, data)| (*urid, data.as_ref()))
        }

        unsafe extern "C" fn extern_retrieve(
            handle: sys::LV2_State_Handle,
            key: u32,
            size: *mut usize,
            type_: *mut u32,
            flags: *mut u32,
        ) -> *const c_void {
            if !flags.is_null() {
                *flags = sys::LV2_State_Flags_LV2_STATE_IS_POD
                    | sys::LV2_State_Flags_LV2_STATE_IS_PORTABLE;
            }

            let handle = (handle as *mut Self).as_mut().unwrap();
            let key = URID::new(key).unwrap();
            if let Some((type_urid, data)) = handle.retrieve(key) {
                *size = data.len();
                *type_ = type_urid.get();
                data.as_ptr() as *const c_void
            } else {
                std::ptr::null()
            }
        }
    }

    fn store(storage: &mut Storage, urids: &AtomURIDCache) {
        let store_fn = Some(
            Storage::extern_store
                as unsafe extern "C" fn(
                    *mut std::ffi::c_void,
                    u32,
                    *const std::ffi::c_void,
                    usize,
                    u32,
                    u32,
                ) -> u32,
        );

        let handle = storage as *mut Storage as *mut c_void;
        let mut store_handle = RawStoreHandle::new(store_fn, handle);
        store_handle
            .draft(URID::new(1).unwrap())
            .init(urids.int, 17)
            .unwrap();
        store_handle
            .draft(URID::new(2).unwrap())
            .init(urids.float, 1.0)
            .unwrap();

        store_handle.commit(URID::new(1).unwrap()).unwrap().unwrap();

        let mut vector_writer = store_handle.draft(URID::new(3).unwrap());
        let mut vector_writer = vector_writer.init(urids.vector(), urids.int).unwrap();
        vector_writer.append(&[1, 2, 3, 4]).unwrap();

        store_handle.commit_all().unwrap();

        store_handle
            .draft(URID::new(4).unwrap())
            .init(urids.int, 0)
            .unwrap();
    }

    fn retrieve(storage: &mut Storage, urids: &AtomURIDCache) {
        let retrieve_fn = Some(
            Storage::extern_retrieve
                as unsafe extern "C" fn(
                    handle: sys::LV2_State_Handle,
                    key: u32,
                    size: *mut usize,
                    type_: *mut u32,
                    flags: *mut u32,
                ) -> *const c_void,
        );
        let handle = storage as *mut Storage as *mut c_void;
        let retrieve_handle = RawRetrieveHandle::new(retrieve_fn, handle);

        assert_eq!(
            17,
            retrieve_handle
                .retrieve(URID::new(1).unwrap())
                .unwrap()
                .read(urids.int, ())
                .unwrap()
        );
        assert_eq!(
            1.0,
            retrieve_handle
                .retrieve(URID::new(2).unwrap())
                .unwrap()
                .read(urids.float, ())
                .unwrap()
        );
        assert_eq!(
            [1, 2, 3, 4],
            retrieve_handle
                .retrieve(URID::new(3).unwrap())
                .unwrap()
                .read(urids.vector(), urids.int)
                .unwrap()
        );
        assert!(retrieve_handle.retrieve(URID::new(4).unwrap()).is_none());
    }

    #[test]
    fn test_storage() {
        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = AtomURIDCache::from_map(&map).unwrap();

        let mut storage = Storage::new();

        store(&mut storage, &urids);

        for (key, (type_, value)) in storage.items.iter() {
            match key.get() {
                1 => {
                    assert_eq!(urids.int, *type_);
                    assert_eq!(17, unsafe { *(value.as_slice() as *const _ as *const i32) });
                }
                2 => {
                    assert_eq!(urids.float, *type_);
                    assert_eq!(1.0, unsafe {
                        *(value.as_slice() as *const _ as *const f32)
                    });
                }
                3 => {
                    assert_eq!(urids.vector::<Int>(), *type_);
                    let space = Space::from_slice(value.as_slice());
                    let data = Vector::read(space, urids.int).unwrap();
                    assert_eq!([1, 2, 3, 4], data);
                }
                _ => panic!("Invalid key!"),
            }
        }

        retrieve(&mut storage, &urids);
    }
}

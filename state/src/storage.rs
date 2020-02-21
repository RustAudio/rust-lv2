use crate::raw::{RetrieveHandle, StoreHandle};
use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use urid::prelude::*;

/// A simple property store.
///
/// This is mostly used to test this crate, but can be used to store properties too. It contains a map from property URIDs to a tuple of a type URID and a vector of bytes. You can access this map by dereferencing the storage.
///
/// You can also directly create [`StoreHandle`s](struct.StoreHandle.html) and [`RetrieveHandle`s](struct.RetrieveHandle.html) that access the storage.
pub struct Storage {
    items: HashMap<URID, (URID, Vec<u8>)>,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}

impl Storage {
    /// Store a property.
    pub fn store(&mut self, key: URID, type_: URID, value: &[u8]) {
        self.items.insert(key, (type_, value.to_owned()));
    }

    /// External version of [`store`](#method.store).
    ///
    /// This function has the appropriate signature to be used as a storage callback.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it dereferences raw pointers.
    ///
    /// The `handle` has to be a pointer to a `Storage` instance and `value` must point to a slice of bytes with the length of `size`.
    pub unsafe extern "C" fn extern_store(
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

    /// Create a `StoreHandle` that saves it's properties to this storage.
    pub fn store_handle(&mut self) -> StoreHandle {
        StoreHandle::new(Some(Self::extern_store), self as *mut Self as *mut c_void)
    }

    /// Try to retrieve a property.
    ///
    /// If the property doesn't exist, `None` is returned.
    pub fn retrieve(&self, key: URID) -> Option<(URID, &[u8])> {
        self.items
            .get(&key)
            .map(|(urid, data)| (*urid, data.as_ref()))
    }

    /// External version of [`retrieve`](#method.retrieve).
    ///
    /// This function has the appropriate signature to be used as a storage callback.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it dereferences raw pointers.
    ///
    /// The `handle` has to be a pointer to a `Storage` instance and `size`, `type_` and `flags` must be valid pointers to instances of their respective types.
    pub unsafe extern "C" fn extern_retrieve(
        handle: sys::LV2_State_Handle,
        key: u32,
        size: *mut usize,
        type_: *mut u32,
        flags: *mut u32,
    ) -> *const c_void {
        if !flags.is_null() {
            *flags =
                sys::LV2_State_Flags_LV2_STATE_IS_POD | sys::LV2_State_Flags_LV2_STATE_IS_PORTABLE;
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

    /// Create a `RetrieveHandle` that retrieves the properties from this storage.
    pub fn retrieve_handle(&mut self) -> RetrieveHandle {
        RetrieveHandle::new(
            Some(Self::extern_retrieve),
            self as *mut Self as *mut c_void,
        )
    }
}

impl Deref for Storage {
    type Target = HashMap<URID, (URID, Vec<u8>)>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for Storage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

use crate::raw::{RawRetrieveHandle, RawStoreHandle};
use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use urid::prelude::*;

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
    pub fn store(&mut self, key: URID, type_: URID, value: &[u8]) {
        self.items.insert(key, (type_, value.to_owned()));
    }

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

    pub fn store_handle(&mut self) -> RawStoreHandle {
        unsafe { RawStoreHandle::new(Some(Self::extern_store), self as *mut Self as *mut c_void) }
    }

    pub fn retrieve(&self, key: URID) -> Option<(URID, &[u8])> {
        self.items
            .get(&key)
            .map(|(urid, data)| (*urid, data.as_ref()))
    }

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

    pub fn retrieve_handle(&mut self) -> RawRetrieveHandle {
        unsafe {
            RawRetrieveHandle::new(
                Some(Self::extern_retrieve),
                self as *mut Self as *mut c_void,
            )
        }
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

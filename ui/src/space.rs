use lv2_atom as atom;

use atom::prelude::*;

/// Smart pointer in the style of lv2_atom::space to be used to
/// communicate between Plugin <-> UI
///
pub struct SelfAllocatingSpace {
    data: Vec<u8>,
    already_read: bool,
}

impl SelfAllocatingSpace {
    pub fn new() -> Self {
        SelfAllocatingSpace {
            data: Vec::new(),
            already_read: false,
        }
    }

    pub unsafe fn put_buffer(&mut self, buffer: std::ptr::NonNull<std::ffi::c_void>, size: usize) {
        self.data.set_len(0);
        self.data.reserve(size);
        std::ptr::copy_nonoverlapping(
            buffer.cast().as_ptr() as *const u8,
            self.data.as_mut_ptr(),
            size,
        );
        self.data.set_len(size);
        self.already_read = false;
    }

    pub fn take(&mut self) -> Option<atom::space::Space> {
        if self.data.is_empty() || self.already_read {
            return None;
        }
        let space = atom::space::Space::from_slice(&self.data);
        self.already_read = true;
        Some(space)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_ptr(&self) -> *const std::ffi::c_void {
        self.data.as_ptr() as *const std::ffi::c_void
    }
}

impl<'a> MutSpace<'a> for SelfAllocatingSpace {
    fn allocate(&mut self, size: usize, apply_padding: bool) -> Option<(usize, &'a mut [u8])> {
        let padding = if apply_padding {
            (8 - self.data.len() % 8) % 8
        } else {
            0
        };
        self.data.resize(self.data.len() + padding, 0);
        let start_point = self.data.len();
        self.data.resize(start_point + size, 0);
        let return_slice = &mut self.data[start_point..];
        Some((padding, unsafe {
            std::slice::from_raw_parts_mut(return_slice.as_mut_ptr(), size)
        }))
    }
}

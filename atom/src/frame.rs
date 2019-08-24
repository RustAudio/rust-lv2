use crate::{AtomBody, AtomURIDCache};
use std::cell::Cell;
use std::marker::PhantomData;

pub trait WritingFrame<'a> {
    fn write_raw(&mut self, data: &[u8]) -> Option<&'a mut [u8]>;
}

pub struct RootWritingFrame<'a> {
    space: Cell<Option<&'a mut [u8]>>,
}

impl<'a> RootWritingFrame<'a> {
    pub fn new(space: &'a mut [u8]) -> Self {
        if space.as_ptr() as usize % 8 != 0 {
            panic!("Trying to create an unaligned atom space");
        }
        Self {
            space: Cell::new(Some(space)),
        }
    }
}

impl<'a> WritingFrame<'a> for RootWritingFrame<'a> {
    fn write_raw(&mut self, raw_data: &[u8]) -> Option<&'a mut [u8]> {
        if self.space.get_mut().is_none() {
            return None;
        }
        let space = self.space.replace(None).unwrap();

        if raw_data.len() > space.len() {
            return None;
        }
        let (lower_slice, upper_slice) = space.split_at_mut(raw_data.len());
        lower_slice.copy_from_slice(raw_data);

        let padding = if raw_data.len() % 8 == 0 {
            0
        } else {
            8 - raw_data.len() % 8
        };

        if padding < upper_slice.len() {
            let (_, upper_slice) = upper_slice.split_at_mut(padding);
            self.space.set(Some(upper_slice))
        }
        Some(lower_slice)
    }
}

pub struct AtomWritingFrame<'a, 'b, A>
where
    A: AtomBody + ?Sized,
{
    atom: &'a mut sys::LV2_Atom,
    parent: &'b mut dyn WritingFrame<'a>,
    phantom: PhantomData<A>,
}

impl<'a, 'b, A> WritingFrame<'a> for AtomWritingFrame<'a, 'b, A>
where
    A: AtomBody + ?Sized,
{
    fn write_raw(&mut self, data: &[u8]) -> Option<&'a mut [u8]> {
        let data = self.parent.write_raw(data)?;
        self.atom.size += data.len() as u32;
        Some(data)
    }
}

impl<'a, 'b> dyn WritingFrame<'a> + 'b {
    pub fn write<T: Sized>(&mut self, instance: &T) -> Option<&'a mut T> {
        let size = std::mem::size_of::<T>();
        let input_data =
            unsafe { std::slice::from_raw_parts(instance as *const T as *const u8, size) };

        let output_data = self.write_raw(input_data)?;

        assert_eq!(size, output_data.len());
        unsafe { Some(&mut *(output_data.as_mut_ptr() as *mut T)) }
    }

    pub fn create_atom_frame<'c, A: AtomBody + ?Sized>(
        &'c mut self,
        urids: &AtomURIDCache,
    ) -> Option<AtomWritingFrame<'a, 'c, A>> {
        let atom = sys::LV2_Atom {
            size: 0,
            type_: A::urid(urids).get(),
        };
        let atom: &'a mut sys::LV2_Atom = self.write(&atom)?;
        Some(AtomWritingFrame {
            atom,
            parent: self,
            phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::*;
    use crate::scalar::*;
    use std::mem::{size_of, size_of_val};
    use urid::mapper::URIDMap;
    use urid::URIDCache;

    #[test]
    fn test_root_frame() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let mut frame: RootWritingFrame = RootWritingFrame::new(unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        });

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        match frame.write_raw(test_data.as_slice()) {
            Some(written_data) => assert_eq!(test_data.as_slice(), written_data),
            None => panic!("Writing failed!"),
        }

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let written_atom = (&mut frame as &mut dyn WritingFrame)
            .write(&test_atom)
            .unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);
    }

    #[test]
    fn test_atom_frame() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let mut root: RootWritingFrame = RootWritingFrame::new(unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        });

        let mut urid_map = URIDMap::new().make_map_interface();
        let urids = AtomURIDCache::from_map(&urid_map.map()).unwrap();

        let mut atom_frame: AtomWritingFrame<Int> = (&mut root as &mut dyn WritingFrame)
            .create_atom_frame(&urids)
            .unwrap();

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        let written_data = atom_frame.write_raw(test_data.as_slice()).unwrap();
        assert_eq!(test_data.as_slice(), written_data);
        assert_eq!(atom_frame.atom.size, test_data.len() as u32);

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let borrowed_frame = &mut atom_frame as &mut dyn WritingFrame;
        let written_atom = borrowed_frame.write(&test_atom).unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);
        assert_eq!(
            atom_frame.atom.size as usize,
            test_data.len() + size_of_val(&test_atom)
        );
    }
}

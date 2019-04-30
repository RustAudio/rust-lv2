use std::ptr::NonNull;

pub struct InputSampledData<T: Copy> {
    pointer: NonNull<T>,
    sample_count: u32,
}

impl<T: Copy> InputSampledData<T> {
    #[inline]
    pub unsafe fn new(pointer: NonNull<()>, sample_count: u32) -> Self {
        Self {
            pointer: pointer.cast(),
            sample_count,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { ::std::slice::from_raw_parts(self.pointer.as_ptr(), self.sample_count as usize) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.sample_count as usize
    }

    #[inline]
    pub fn iter(&self) -> ::std::slice::Iter<T> {
        self.as_slice().iter()
    }
}

pub struct OutputSampledData<T: Copy> {
    pointer: NonNull<T>,
    sample_count: u32,
}

impl<T: Copy> OutputSampledData<T> {
    #[inline]
    pub unsafe fn new(pointer: NonNull<()>, sample_count: u32) -> Self {
        Self {
            pointer: pointer.cast(),
            sample_count,
        }
    }

    #[inline]
    pub fn put(&self, value: T, index: usize) {
        let slice = unsafe {
            ::std::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.sample_count as usize)
        };
        slice[index] = value
    }

    #[inline]
    pub fn collect_from<I: IntoIterator<Item = T>>(&self, iterable: I) {
        let slice = unsafe {
            ::std::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.sample_count as usize)
        };

        for (output, input) in slice.iter_mut().zip(iterable) {
            *output = input
        }
    }

    #[inline]
    pub fn fill(&self, value: T) {
        let slice = unsafe {
            ::std::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.sample_count as usize)
        };

        for output in slice.iter_mut() {
            *output = value
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.sample_count as usize
    }
}

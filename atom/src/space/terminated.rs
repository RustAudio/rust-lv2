use crate::space::error::AtomWriteError;
use crate::space::{SpaceAllocator, SpaceWriterSplitAllocation};

/// An helper space writer, that wraps an existing writer and makes sure all writes are
/// terminated with a given terminator byte.
///
/// Further writes overwrite the added terminator byte, and append a new one.
///
/// This helper is useful to implement Atom writer for null-terminated strings, for instance.
///
/// # Example
///
/// ```
/// use lv2_atom::space::{SpaceCursor, SpaceWriter, SpaceAllocator, Terminated};
/// let mut buffer = [0; 20];
/// // Our underlying allocator
/// let cursor = SpaceCursor::new(&mut buffer);
///
/// let mut writer = Terminated::new(cursor, 0x42); // Alternative: use cursor.terminated().
///
/// writer.write_bytes(b"Hello, world!").unwrap();
/// assert_eq!(writer.allocated_bytes(), b"Hello, world!\x42");
///
/// writer.write_bytes(b" Boop!").unwrap();
/// assert_eq!(&buffer, b"Hello, world! Boop!\x42");
///
/// ```
pub struct Terminated<W: SpaceAllocator> {
    inner: W,
    terminator: u8,
    wrote_terminator_byte: bool,
}

impl<W: SpaceAllocator> Terminated<W> {
    /// Creates a new Terminated writer, from an inner writer and a given terminator byte.
    pub fn new(inner: W, terminator: u8) -> Self {
        Self {
            inner,
            terminator,
            wrote_terminator_byte: false,
        }
    }

    /// Unwraps the `Terminated` helper and returns the underlying allocator.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: SpaceAllocator> SpaceAllocator for Terminated<W> {
    fn allocate_and_split(
        &mut self,
        size: usize,
    ) -> Result<SpaceWriterSplitAllocation, AtomWriteError> {
        if self.wrote_terminator_byte {
            // SAFETY: We checked we already wrote the terminator byte, and it is safe to be overwritten
            unsafe { self.inner.rewind(1)? };
        }

        let SpaceWriterSplitAllocation {
            previous,
            allocated,
        } = self.inner.allocate_and_split(size + 1)?;
        allocated[size] = self.terminator;
        self.wrote_terminator_byte = true;

        Ok(SpaceWriterSplitAllocation {
            previous,
            allocated: &mut allocated[..size],
        })
    }

    #[inline]
    unsafe fn rewind(&mut self, byte_count: usize) -> Result<(), AtomWriteError> {
        self.inner.rewind(byte_count)
    }

    #[inline]
    fn allocated_bytes(&self) -> &[u8] {
        self.inner.allocated_bytes()
    }

    #[inline]
    unsafe fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        self.inner.allocated_bytes_mut()
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        self.inner.remaining_bytes()
    }
}

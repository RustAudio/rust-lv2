use crate::space::AllocateSpace;

/// Linked list element for dynamic atom writing.
///
/// This struct works in conjunction with [`SpaceHead`](struct.SpaceHead.html) to provide a way to write atoms to dynamically allocated memory.
pub struct SpaceList {
    next: Option<(Box<Self>, Box<[u8]>)>,
}

impl Default for SpaceList {
    fn default() -> Self {
        Self { next: None }
    }
}

impl SpaceList {
    /// Append an element to the list.
    ///
    /// If this is the last element of the list, allocate a slice of the required length and append a new element to the list. If not, do nothing and return `None`.
    pub fn allocate(&mut self, size: usize) -> Option<(&mut Self, &mut [u8])> {
        if self.next.is_some() {
            return None;
        }

        let new_data = vec![0u8; size].into_boxed_slice();
        let new_element = Box::new(Self::default());
        self.next = Some((new_element, new_data));
        self.next
            .as_mut()
            .map(|(new_element, new_data): &mut (Box<Self>, Box<[u8]>)| {
                (new_element.as_mut(), new_data.as_mut())
            })
    }

    /// Create a vector containing the data from all elements following this one.
    pub fn to_vec(&self) -> Vec<u8> {
        self.iter()
            .map(|slice| slice.iter())
            .flatten()
            .copied()
            .collect()
    }

    /// Return an iterator over the chunks of all elements following this one.
    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        std::iter::successors(self.next.as_ref(), |element| element.0.next.as_ref())
            .map(|(_, data)| data.as_ref())
    }
}

/// A mutable space that dynamically allocates memory.
///
/// This space uses a linked list of [`SpaceElement`s](struct.SpaceElement.html) to allocate memory. Every time `allocate` is called, a new element is appended to the list and a new byte slice is created.
///
/// In order to use this space and retrieve the written data once it was written, you create a `SpaceElement` and create a new head with it. Then, you use the head like any other `MutSpace` and when you're done, you retrieve the written data by either calling [`to_vec`](struct.SpaceElement.html#method.to_vec) or [`iter`](struct.SpaceElement.html#iter).
///
/// # Usage example
///
/// ```
/// # use lv2_core::prelude::*;
/// # use lv2_atom::prelude::*;
/// # use lv2_atom::space::*;
/// # use urid::*;
/// # use std::pin::Pin;
/// # let map = HashURIDMapper::new();
/// // URID cache creation is omitted.
/// let urids: AtomURIDCollection = map.populate_collection().unwrap();
///
/// // Creating the first element in the list and the writing head.
/// let mut element = SpaceList::default();
/// let mut head = SpaceHead::new(&mut element);
///
/// // Writing an integer.
/// (&mut head as &mut dyn MutSpace).init(urids.int, 42).unwrap();
///
/// // Retrieving a continuos vector with the written data and verifying it's contents.
/// let written_data: Vec<u8> = element.to_vec();
/// let atom = unsafe { UnidentifiedAtom::new_unchecked(Space::from_slice(written_data.as_ref())) };
/// assert_eq!(42, atom.read(urids.int, ()).unwrap());
/// ```
pub struct SpaceHead<'a> {
    element: Option<&'a mut SpaceList>,
    allocated_space: usize,
}

impl<'a> SpaceHead<'a> {
    /// Create a new head that references the given element.
    pub fn new(element: &'a mut SpaceList) -> Self {
        Self {
            element: Some(element),
            allocated_space: 0,
        }
    }
}

impl<'a> AllocateSpace<'a> for SpaceHead<'a> {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]> {
        let element = self.element.take()?;
        let (new_element, new_space) = element.allocate(size)?;
        self.element = Some(new_element);
        self.allocated_space += size;
        Some(new_space)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        &[]
    }
}

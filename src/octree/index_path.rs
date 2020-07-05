use std::fmt::Write;
use std::num::NonZeroU64;

// Can represent max 21 layers of structures
// Always prepend index path with a 1
// 0..63 bits: 21 index path, grouped by 3 bits
#[derive(Copy, Clone)]
pub struct IndexPath (NonZeroU64);

impl std::fmt::Debug for IndexPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut current = self.clone();
        while !current.is_empty() {
            f.write_char((current.peek() + '0' as u8).into())?;
            current = current.pop();
            f.write_char('/')?;
        }
        Ok(())
    }
}

impl IndexPath {
    pub const MAX_SIZE: u8 = 21;
    pub fn empty() -> IndexPath {
        unsafe {
            IndexPath(std::num::NonZeroU64::new_unchecked(1))
        }
    }
    pub fn new(octant: u8) -> IndexPath {
        IndexPath::empty().push(octant)
    }
    pub fn is_empty(&self) -> bool {
        self.0.get() == 1
    }
    pub fn is_full(&self) -> bool {
        // Check highest bit
        (self.0.get() >> 63) == 1
    }
    pub fn peek(&self) -> u8 {
        assert!(!self.is_empty());
        self.0.get() as u8 & 0b111
    }
    pub fn pop(&self) -> IndexPath {
        assert!(!self.is_empty());
        unsafe {
            IndexPath(NonZeroU64::new_unchecked(self.0.get() >> 3))
        }
    }
    pub fn push(&self, octant: u8) -> IndexPath {
        assert!(octant < 8);
        assert!(!self.is_full(), "The index path is full");
        unsafe {
            IndexPath(NonZeroU64::new_unchecked((self.0.get() << 3) | (octant as u64)))
        }
    }
    pub fn replace(&self, octant: u8) -> IndexPath {
        assert!(octant < 8);
        unsafe {
            IndexPath(NonZeroU64::new_unchecked((self.0.get() & !0b111) | (octant as u64)))
        }
    }
    pub fn len(&self) -> u8 {
        let num_empty_slots = self.0.get().leading_zeros() as u8 / 3;
        Self::MAX_SIZE - num_empty_slots
    }
}

#[cfg(test)]
mod tests {
    use crate::octree::index_path::IndexPath;
    use std::mem::size_of;

    #[test]
    fn test_index_path() {
        assert_eq!(size_of::<IndexPath>(), size_of::<u64>());
        assert_eq!(size_of::<Option<IndexPath>>(), size_of::<u64>());

        let mut path = IndexPath::empty();
        for i in 0..IndexPath::MAX_SIZE {
            assert_eq!(path.len(), i);
            path = path.push(0);
        }
        assert_eq!(path.len(), IndexPath::MAX_SIZE);
    }
}

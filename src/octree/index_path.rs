use std::fmt::Write;
use std::num::NonZeroU64;
use amethyst::core::math::Vector3;
use super::direction::Direction;

// Can represent max 21 layers of structures
// Always prepend index path with a 1
// 0..63 bits: 21 index path, grouped by 3 bits
#[derive(Copy, Clone)]
pub struct IndexPath (NonZeroU64);

impl std::fmt::Debug for IndexPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut current = self.clone();
        while !current.is_empty() {
            f.write_char((current.peek() as u8 + '0' as u8).into())?;
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
    pub fn new(octant: Direction) -> IndexPath {
        IndexPath::empty().push(octant)
    }
    pub fn is_empty(&self) -> bool {
        self.0.get() == 1
    }
    pub fn is_full(&self) -> bool {
        // Check highest bit
        (self.0.get() >> 63) == 1
    }
    pub fn peek(&self) -> Direction {
        assert!(!self.is_empty());
        (self.0.get() as u8 & 0b111).into()
    }
    pub fn pop(&self) -> IndexPath {
        assert!(!self.is_empty());
        unsafe {
            IndexPath(NonZeroU64::new_unchecked(self.0.get() >> 3))
        }
    }
    pub fn push(&self, octant: Direction) -> IndexPath {
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

impl PartialEq for IndexPath {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IndexPath {
}

impl Iterator for IndexPath {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let dir = self.peek();
            self.0 = self.pop().0;
            Some(dir)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IndexPath;
    use super::Direction;
    use std::mem::size_of;

    #[test]
    fn test_index_path() {
        assert_eq!(size_of::<IndexPath>(), size_of::<u64>());
        assert_eq!(size_of::<Option<IndexPath>>(), size_of::<u64>());

        let mut path = IndexPath::empty();
        for i in 0..IndexPath::MAX_SIZE {
            assert_eq!(path.len(), i);
            path = path.push(Direction::FrontLeftBottom);
        }
        assert_eq!(path.len(), IndexPath::MAX_SIZE);
    }

    #[test]
    fn test_iterator() {
        let mut index_path = IndexPath::empty();
        for i in 0..7 {
            index_path = index_path.push(i.into());
        }
        for i in (0..7).rev() {
            let dir: Direction = i.into();
            assert_eq!(index_path.next(), Some(dir));
        }

        assert_eq!(index_path.next(), None);
    }
}

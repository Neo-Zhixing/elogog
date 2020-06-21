use super::node::Node;
use std::ops::{Index, IndexMut};

pub(super) struct Arena<const GROUP_SIZE: usize> {
    nodes: [std::mem::MaybeUninit<[Node; GROUP_SIZE]>; 256],
    freemask: [u64; 256 / 64], // 1 when free, 0 when used
    // Invariant: the location in nodes pointed by next_available is always available
    next_available: Option<u8>,
}

impl<const GROUP_SIZE: usize> Arena<GROUP_SIZE> {
    pub(super) fn new() -> Arena<GROUP_SIZE> {
        let nodes: [std::mem::MaybeUninit<[Node; GROUP_SIZE]>; 256] = std::mem::MaybeUninit::uninit_array();
        Arena {
            nodes,
            freemask: [std::u64::MAX; 4],
            next_available: Some(0),
        }
    }
    pub fn is_full(&self) -> bool {
        self.next_available.is_none()
    }
    pub(super) fn available_at(&self, index: u8) -> bool {
        let val = self.freemask[(index >> 6) as usize];
        let subindex = index & 0b111111;
        val & ((1 as u64) << (subindex as u64)) != 0
    }
    pub(super) fn set_available_at(&mut self, index: u8, available: bool) {
        let entry = &mut self.freemask[(index >> 6) as usize];
        let subindex = index & 0b111111;
        if available {
            *entry |= ((1 as u64) << (subindex as u64));
        } else {
            *entry &= !((1 as u64) << (subindex as u64));
        }
    }

    pub(super) fn free(&mut self, index: u8) {
        self.set_available_at(index, true);
    }
    fn find_and_set_next_available(&mut self) {
        for (index, mask) in self.freemask.iter().enumerate() {
            if *mask == 0 {
                // All zeros. This mask is full. Skip.
                continue;
            }
            let trailing_zeros = mask.trailing_zeros() as u8;
            self.next_available = Some(trailing_zeros + ((index as u8) << 6));
            return;
        }
        self.next_available = None;
    }
    pub(super) fn alloc(&mut self) -> Option<(&mut [Node; GROUP_SIZE], u8)> {
        if let Some(next_available) = self.next_available {
            if next_available < std::u8::MAX && self.available_at(next_available + 1) {
                self.next_available = Some(next_available + 1);
            } else {
                // Finding and set the next available spot
                self.find_and_set_next_available();
            }

            self.set_available_at(next_available, false);
            let node_group = unsafe {self.nodes[next_available as usize].get_mut() };
            for i in  node_group.iter_mut() {
                *i = Default::default();
            }
            Some((node_group, next_available))
        } else {
            None
        }
    }
}

impl<const GROUP_SIZE: usize> Index<u8> for Arena<GROUP_SIZE> {
    type Output = [Node; GROUP_SIZE];
    fn index(&self, i: u8) -> &[Node; GROUP_SIZE] {
        if self.available_at(i) {
            // The memory can be assumed initialized because the initialized flag was set
            unsafe { self.nodes[i as usize].get_ref() }
        } else {
            panic!("Arena<{}>: Accessing unallocated node group", GROUP_SIZE)
        }
    }
}

impl<const GROUP_SIZE: usize> IndexMut<u8> for Arena<GROUP_SIZE>  {
    fn index_mut(&mut self, i: u8) -> &mut [Node; GROUP_SIZE] {
        if self.available_at(i) {
            unsafe { self.nodes[i as usize].get_mut()  }
        } else {
            panic!("Arena<{}>: Accessing unallocated node group", GROUP_SIZE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Arena;

    #[test]
    fn test_available_at() {
        let mut arena: Arena<8> = Arena::new();
        for i in 0 .. std::u8::MAX {
            assert_eq!(arena.available_at(i), true);
        }
        arena.set_available_at(18, false);
        assert_eq!(arena.available_at(18), false);

        assert_eq!(arena.available_at(182), true);
        arena.set_available_at(182, false);
        assert_eq!(arena.available_at(182), false);
        arena.set_available_at(182, true);
        assert_eq!(arena.available_at(182), true);
    }

    #[test]
    fn test_find_and_set_next_available() {
        let mut arena: Arena<8> = Arena::new();
        arena.set_available_at(0, false);
        arena.set_available_at(1, false);
        arena.set_available_at(3, false);
        arena.set_available_at(4, false);
        arena.find_and_set_next_available();
        assert_eq!(arena.next_available, Some(2));
        assert_eq!(arena.is_full(), false);

        // Block off a range of slots
        for i in 0..120 {
            arena.set_available_at(i, false);
        }
        arena.set_available_at(121, false);
        arena.set_available_at(183, false);
        arena.find_and_set_next_available();

        assert_eq!(arena.next_available, Some(120));
        assert_eq!(arena.is_full(), false);


        // All full
        for i in 0..=std::u8::MAX {
            arena.set_available_at(i, false);
        }
        arena.find_and_set_next_available();
        assert_eq!(arena.is_full(), true);
        assert_eq!(arena.next_available, None);
    }
}

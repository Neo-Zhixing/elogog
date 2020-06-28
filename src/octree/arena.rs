use super::voxel::Voxel;
use std::ops::{Index, IndexMut};

type ArenaSegmentIndice = u8;

#[derive(Copy, Clone, Debug, Default)]
pub struct ArenaRangeIndice {
    segment: u8,
    indice: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct ArenaNodeIndice {
    range: ArenaRangeIndice,
    index: u8 // The position in the child list
}

pub struct ArenaNode {
    children_index: ArenaRangeIndice, // meaningless when all leaf
    leaf_mask: u8, // 1 for child, 0 for leaf
    load_mask: u8,
    data: [Voxel; 8],
}

impl ArenaNode {
    pub fn has_child_on_dir(&self, dir: u8) -> bool {
        (1 << dir) & self.leaf_mask != 0
    }
    pub fn child_on_dir(&self, dir: u8) -> Option<ArenaNodeIndice> {
        if self.has_child_on_dir(dir) {
            Some(ArenaNodeIndice {
                range: self.children_index,
                index: (self.leaf_mask >> (dir + 1)).count_ones() as u8,
            })
        } else {
            None
        }

    }
}

const ARENA_BLOCK_SIZE: usize = 256;
struct ArenaSegment {
    nodes: *mut ArenaNode,
    freemask: [u64; ARENA_BLOCK_SIZE / 64], // 1 when free, 0 when used
    // Invariant: the location in nodes pointed by next_available is always available
    next_available: Option<u8>,
    group_size: u8,
}



impl ArenaSegment {
    fn new(group_size: u8) -> ArenaSegment {
        assert!(1 <= group_size && group_size <= 8);
        let (layout, block_size) = std::alloc::Layout::new::<ArenaNode>()
            .repeat((group_size as usize) * ARENA_BLOCK_SIZE).unwrap();
        assert_eq!(block_size, std::mem::size_of::<ArenaNode>());
        ArenaSegment {
            nodes: unsafe { std::alloc::alloc(layout) as *mut ArenaNode },
            freemask: [std::u64::MAX; 4],
            next_available: Some(0),
            group_size
        }
    }
    fn is_full(&self) -> bool {
        self.next_available.is_none()
    }
    fn available_at(&self, index: ArenaSegmentIndice) -> bool {
        let val = self.freemask[(index >> 6) as usize];
        let subindex = index & 0b111111;
        val & ((1 as u64) << (subindex as u64)) != 0
    }
    fn set_available_at(&mut self, index: ArenaSegmentIndice, available: bool) {
        let entry = &mut self.freemask[(index >> 6) as usize];
        let subindex = index & 0b111111; // Take the last 6 bits;
        if available {
            *entry |= ((1 as u64) << (subindex as u64));
        } else {
            *entry &= !((1 as u64) << (subindex as u64));
        }
    }

    fn free(&mut self, index: ArenaSegmentIndice) {
        self.set_available_at(index, true);
    }
    fn find_next_available(&mut self) -> Option<u8> {
        for (index, mask) in self.freemask.iter().enumerate() {
            if *mask == 0 {
                // All zeros. This mask is full. Skip.
                continue;
            }
            let trailing_zeros = mask.trailing_zeros() as u8;
            return Some(trailing_zeros + ((index as u8) << 6));
        }
        None
    }
    fn alloc(&mut self) -> ArenaSegmentIndice {
        if let Some(mut next_available) = self.next_available {
            // Move forward next_available
            self.set_available_at(next_available, false);
            if next_available < std::u8::MAX && self.available_at(next_available + 1) {
                // First, check the next slot because it is likely to be empty
                self.next_available = Some(next_available + 1);
            } else {
                // Otherwise, find and set the next available spot
                self.next_available = self.find_next_available();
            }

            // Initialize variables
            let node_group = unsafe {
                let ptr = self.nodes.offset((next_available as isize) * (self.group_size as isize));
                std::slice::from_raw_parts_mut(ptr, self.group_size as usize)
            };
            for i in node_group.iter_mut() {
                *i = ArenaNode {
                    children_index: ArenaRangeIndice { segment: 0, indice: 0 },
                    leaf_mask: 0,
                    load_mask: 0,
                    data: Default::default(),
                };
            }
            next_available
        } else {
            panic!("Can't allocate new node group because the current segment is full");
        }
    }
}

impl Index<ArenaSegmentIndice> for ArenaSegment {
    type Output = [ArenaNode];
    fn index(&self, i: ArenaSegmentIndice) -> &[ArenaNode] {
        if !self.available_at(i) {
            // The memory can be assumed initialized because the initialized flag was set
            unsafe {
                let ptr = self.nodes.offset((i as isize) * (self.group_size as isize));
                std::slice::from_raw_parts(ptr, self.group_size as usize)
            }
        } else {
            panic!("Arena<{}>: Accessing unallocated node group", self.group_size)
        }
    }
}

impl IndexMut<ArenaSegmentIndice> for ArenaSegment  {
    fn index_mut(&mut self, i: ArenaSegmentIndice) -> &mut [ArenaNode] {
        if !self.available_at(i) {
            unsafe {
                let ptr = self.nodes.offset((i as isize) * (self.group_size as isize));
                std::slice::from_raw_parts_mut(ptr, self.group_size as usize)
            }
        } else {
            panic!("Arena<{}>: Accessing unallocated node group", self.group_size)
        }
    }
}

impl Drop for ArenaSegment {
    fn drop(&mut self) {
        let (layout, block_size) = std::alloc::Layout::new::<ArenaNode>()
            .repeat((self.group_size as usize) * ARENA_BLOCK_SIZE).unwrap();
        unsafe {
            std::alloc::dealloc(self.nodes as *mut u8, layout);
        }
    }
}

pub struct Arena {
    segments: [Vec<ArenaSegment>; 8]
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            segments: Default::default(),
        }
    }
    fn alloc(&mut self, group_size: u8) -> ArenaRangeIndice {
        assert!(0 < group_size && group_size <= 8);

        // Finding in reverse order the first segment that is empty.
        // In reverse order because empty segments are more likely to be on the back
        let segment_range = &mut self.segments[group_size as usize];
        if let Some((index, first_empty_segment)) =
            segment_range
                .iter_mut()
                .enumerate()
                .rev()
                .find(|(_, segment)| !segment.is_full()) {
            let indice = first_empty_segment.alloc();
            ArenaRangeIndice {
                segment: index as u8,
                indice,
            }
        } else if segment_range.len() <= 255 {
            let mut new_segment = ArenaSegment::new(group_size);
            let indice = new_segment.alloc();
            segment_range.push(new_segment);
            ArenaRangeIndice {
                segment: (segment_range.len() - 1) as u8,
                indice,
            }
        } else {
            panic!("Arena Overflow");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ArenaSegment;
    use super::Arena;
    use super::ArenaNode;

    #[test]
    fn test_available_at() {
        let mut arena: ArenaSegment = ArenaSegment::new(8);
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
        let mut arena: ArenaSegment = ArenaSegment::new(8);
        arena.set_available_at(0, false);
        arena.set_available_at(1, false);
        arena.set_available_at(3, false);
        arena.set_available_at(4, false);
        assert_eq!(arena.find_next_available(), Some(2));
        assert_eq!(arena.is_full(), false);

        // Block off a range of slots
        for i in 0..120 {
            arena.set_available_at(i, false);
        }
        arena.set_available_at(121, false);
        arena.set_available_at(183, false);

        assert_eq!(arena.find_next_available(), Some(120));
        assert_eq!(arena.is_full(), false);


        // All full
        for i in 0..=std::u8::MAX {
            arena.set_available_at(i, false);
        }
        assert_eq!(arena.is_full(), true);
        assert_eq!(arena.find_next_available(), None);
    }

    #[test]
    fn test_arena_alloc() {
        let mut arena = Arena::new();
        for j in 0..=255 {
            for i in 0..=255 {
                let indice = arena.alloc(1);
                assert_eq!(indice.segment, j);
                assert_eq!(indice.indice, i);
            }
            assert!(arena.segments[1][j as usize].is_full())
        }
    }

    #[test]
    fn test_child_on_dir() {
        let node = ArenaNode {
            children_index: Default::default(), // meaningless when all leaf
            leaf_mask: 0b00101101, // 1 for child, 0 for leaf
            load_mask: 0,
            data: Default::default(),
        };
        assert_eq!(node.child_on_dir(0).unwrap().index, 3);
        assert_eq!(node.child_on_dir(2).unwrap().index, 2);
        assert_eq!(node.child_on_dir(3).unwrap().index, 1);
        assert_eq!(node.child_on_dir(5).unwrap().index, 0);
        assert!(node.child_on_dir(1).is_none());
        assert!(node.child_on_dir(4).is_none());
        assert!(node.child_on_dir(6).is_none());
        assert!(node.child_on_dir(7).is_none());
    }
}

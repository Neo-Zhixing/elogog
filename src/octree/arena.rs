use super::voxel::Voxel;
use std::ops::{Index, IndexMut};
use std::fmt::Write;

// Locate a node block inside a segment
type ArenaSegmentIndice = u8;

// Locate a node group inside the entire arena
#[derive(Copy, Clone, Debug, Default)]
pub struct ArenaBlockIndice {
    segment: u8,
    indice: u8,
    block_size: u8,
}

impl ArenaBlockIndice {
    pub fn child(&self, index: u8) -> ArenaNodeIndice {
        ArenaNodeIndice {
            block: self.clone(),
            index,
        }
    }
}

// Locate a single voxel inside the entire arena
#[derive(Copy, Clone, Debug)]
pub struct ArenaNodeIndice {
    block: ArenaBlockIndice,
    index: u8 // The position in the child list
}

// A node in the octree. Always group by N where N is parent's child count
#[derive(Clone)]
pub struct ArenaNode {
    children_index_segment: u8, // meaningless when all leaf
    children_index_indice: u8,
    pub(super) leaf_mask: u8, // 1 for child, 0 for leaf
    load_mask: u8,
    pub(super) data: [Voxel; 8],
}

impl ArenaNode {
    #[inline]
    pub fn has_child_on_dir(&self, dir: u8) -> bool {
        (1 << dir) & self.leaf_mask != 0
    }

    pub fn child_on_dir(&self, dir: u8) -> Option<ArenaNodeIndice> {
        if self.has_child_on_dir(dir) {
            Some(ArenaNodeIndice {
                block: self.children_block(),
                index: if dir == 7 { 0 } else { (self.leaf_mask >> (dir + 1)).count_ones() as u8 },
            })
        } else {
            None
        }

    }

    #[inline]
    pub fn num_children(&self) -> u8 {
        self.leaf_mask.count_ones() as u8
    }

    #[inline]
    pub fn children_block(&self) -> ArenaBlockIndice {
        ArenaBlockIndice {
            segment: self.children_index_segment,
            indice: self.children_index_indice,
            block_size: self.num_children(),
        }
    }
    #[inline]
    pub fn set_on_dir(&mut self, dir: u8, voxel: Voxel) {
        debug_assert!(dir < 8);
        self.data[dir as usize] = voxel;
    }

    #[inline]
    pub fn is_leaf_node(&self) -> bool {
        self.leaf_mask == 0 // All bits 0
    }
    pub fn is_condensable(&self) -> bool {
        if !self.is_leaf_node() {
            return false;
        }
        let com = self.data[0];
        self.data.iter().all(|x| *x == com)
    }

    fn print_node(&self, f: &mut std::fmt::Formatter<'_>, dir: u8) -> Result<(), std::fmt::Error> {
        debug_assert!(dir < 8);
        if self.has_child_on_dir(dir) {
            write!(f, "\x1b[0;31m{:?}\x1b[0m", self.data[dir as usize])?;
        } else {
            std::fmt::Debug::fmt(&self.data[dir as usize], f)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for ArenaNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("|---DN---|---UP---|\n")?;

        f.write_str("| ")?;
        self.print_node(f, 2)?;
        f.write_str("  ")?;
        self.print_node(f, 3)?;
        f.write_str(" | ")?;
        self.print_node(f, 6)?;
        f.write_str("  ")?;
        self.print_node(f, 7)?;
        f.write_str(" |\n")?;

        f.write_str("| ")?;
        self.print_node(f, 0)?;
        f.write_str("  ")?;
        self.print_node(f, 1)?;
        f.write_str(" | ")?;
        self.print_node(f, 4)?;
        f.write_str("  ")?;
        self.print_node(f, 5)?;
        f.write_str(" |\n-------------------\n")?;
        Ok(())
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
        debug_assert!(1 <= group_size && group_size <= 8);
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

    #[inline]
    fn count_nodes(&self) -> usize {
        self.freemask.iter().map(|num| num.count_zeros() as usize).sum()
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.next_available.is_none()
    }

    #[inline]
    fn available_at(&self, index: ArenaSegmentIndice) -> bool {
        let val = self.freemask[(index >> 6) as usize];
        let subindex = index & 0b111111;
        val & ((1 as u64) << (subindex as u64)) != 0
    }

    #[inline]
    fn set_available_at(&mut self, index: ArenaSegmentIndice, available: bool) {
        let entry = &mut self.freemask[(index >> 6) as usize];
        let subindex = index & 0b111111; // Take the last 6 bits;
        if available {
            *entry |= (1 as u64) << (subindex as u64);
        } else {
            *entry &= !((1 as u64) << (subindex as u64));
        }
    }

    #[inline]
    fn free(&mut self, index: ArenaSegmentIndice) {
        debug_assert!(!self.available_at(index), "Double free");
        self.set_available_at(index, true);
        self.next_available = Some(index);
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
        if let Some(next_available) = self.next_available {
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

impl IndexMut<ArenaSegmentIndice> for ArenaSegment {
    fn index_mut(&mut self, i: ArenaSegmentIndice) -> &mut [ArenaNode] {
        if !self.available_at(i) {
            // The memory can be assumed initialized because the initialized flag was set
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
        let (layout, _) = std::alloc::Layout::new::<ArenaNode>()
            .repeat((self.group_size as usize) * ARENA_BLOCK_SIZE).unwrap();
        unsafe {
            std::alloc::dealloc(self.nodes as *mut u8, layout);
        }
    }
}
unsafe impl Send for ArenaSegment { }
unsafe impl Sync for ArenaSegment { }

impl std::fmt::Debug for ArenaSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for i in 0..=255 {
            if self.available_at(i) {
                f.write_char('X')?;
            } else {
                f.write_char('o')?;
            }
        }
        f.write_char('\n')?;
        if let Some(next_available) = self.next_available {
            for i in 0..=255 {
                if i == next_available {
                    f.write_char('^')?;
                } else {
                    f.write_char(' ')?;
                }
            }
        }
        f.write_char('\n')?;
        writeln!(f, "Each slot has {} nodes", self.group_size)?;
        Ok(())
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
    pub fn alloc(&mut self, block_size: u8) -> ArenaBlockIndice {
        debug_assert!(0 < block_size && block_size <= 8);

        // Finding in reverse order the first segment that is empty.
        // In reverse order because empty segments are more likely to be on the back
        let segment_range = &mut self.segments[block_size as usize - 1];
        if let Some((index, first_empty_segment)) =
            segment_range
                .iter_mut()
                .enumerate()
                .rev()
                .find(|(_, segment)| !segment.is_full()) {
            let indice = first_empty_segment.alloc();
            ArenaBlockIndice {
                segment: index as u8,
                indice,
                block_size
            }
        } else if segment_range.len() <= 255 {
            let mut new_segment = ArenaSegment::new(block_size);
            let indice = new_segment.alloc();
            segment_range.push(new_segment);
            ArenaBlockIndice {
                segment: (segment_range.len() - 1) as u8,
                indice,
                block_size
            }
        } else {
            panic!("Arena Overflow");
        }
    }

    pub fn free(&mut self, indice: ArenaBlockIndice) {
        if indice.block_size > 0 {
            self.segments[indice.block_size as usize - 1][indice.segment as usize].free(indice.indice);
        }
    }

    pub fn realloc(&mut self, indice: ArenaNodeIndice, freemask: u8) {
        if freemask == 0 {
            // Free everything. And skip the trouble of copying, etc.
            let node = self.get_node(indice);
            let old_block_indice = node.children_block();
            self.free(old_block_indice);


            let node = self.get_node_mut(indice);
            node.leaf_mask = 0;
            node.children_index_indice = 0;
            node.children_index_segment = 0;
            return;
        }
        let new_block_size = freemask.count_ones() as u8;
        let new_block_indice = self.alloc(new_block_size);
        let node = self.get_node(indice);
        let old_block_indice = node.children_block();
        if  node.leaf_mask != 0 { // Skip if the old freemask is all zero - nothing to copy
            let old_block_size = node.num_children();
            let mut old_freemask = node.leaf_mask;
            let mut new_freemask = freemask;
            let mut old_block_index = 1;
            let mut new_block_index = 1;
            for _ in 0..8 {
                let old_block_has = old_freemask & 0b1 == 1;
                old_freemask = old_freemask >> 1;
                let new_block_has = new_freemask & 0b1 == 1;
                new_freemask = new_freemask >> 1;
                if old_block_has && new_block_has {
                    self.get_block_mut(new_block_indice)[(new_block_size - new_block_index) as usize] =
                        self.get_block(old_block_indice)[(old_block_size - old_block_index) as usize].clone();
                    old_block_index += 1;
                    new_block_index += 1;
                } else if old_block_has {
                    old_block_index += 1;
                } else if new_block_has {
                    new_block_index += 1;
                }
                if old_freemask == 0 || new_freemask == 0 {
                    // If either is 0, there is no need to proceed.
                    break;
                }
            }
        }
        // Copy blocks into new space
        let node = self.get_node_mut(indice);
        node.leaf_mask = freemask;
        node.children_index_indice = new_block_indice.indice;
        node.children_index_segment = new_block_indice.segment;
        self.free(old_block_indice);
    }

    pub fn get_block(&self, indice: ArenaBlockIndice) -> &[ArenaNode] {
        if indice.block_size == 0 {
            &[]
        } else {
            &self.segments[indice.block_size as usize - 1][indice.segment as usize][indice.indice]
        }
    }

    #[inline]
    pub fn get_node(&self, indice: ArenaNodeIndice) -> &ArenaNode {
        &self.get_block(indice.block)[indice.index as usize]
    }

    #[inline]
    pub fn get_block_mut(&mut self, indice: ArenaBlockIndice) -> &mut [ArenaNode] {
        if indice.block_size == 0 {
            &mut []
        } else {
            &mut self.segments[indice.block_size as usize - 1][indice.segment as usize][indice.indice]
        }
    }

    #[inline]
    pub fn get_node_mut(&mut self, indice: ArenaNodeIndice) -> &mut ArenaNode {
        &mut self.get_block_mut(indice.block)[indice.index as usize]
    }

    #[inline]
    pub fn count_nodes(&self) -> usize {
        self.segments.iter().map(|segment| segment.into_iter().map(|d| d.count_nodes()).sum::<usize>()).sum()
    }
}

impl std::fmt::Debug for Arena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("--------- Chunk Arena ----------\n")?;
        for (index, segments) in self.segments.iter().enumerate() {
            writeln!(f, "-----Block sized {}-----", index + 1)?;
            for (index, segment) in segments.iter().enumerate() {
                writeln!(f, "{}: ", index)?;
                segment.fmt(f)?;
            }
        }
        f.write_str("------- End Chunk Arena --------\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ArenaSegment;
    use super::Arena;
    use super::ArenaNode;
    use std::mem::size_of;

    #[test]
    fn test_size_of() {
        assert_eq!(size_of::<ArenaNode>(), size_of::<u8>()*4 + size_of::<u16>()*8);
    }

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

        // Block off a range of slots
        for i in 0..120 {
            arena.set_available_at(i, false);
        }
        arena.set_available_at(121, false);
        arena.set_available_at(183, false);

        assert_eq!(arena.find_next_available(), Some(120));


        // All full
        for i in 0..=std::u8::MAX {
            arena.set_available_at(i, false);
        }
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
            assert!(arena.segments[0][j as usize].is_full());
        }
    }

    #[test]
    fn test_child_on_dir() {
        let node = ArenaNode {
            children_index_segment: 0, // meaningless when all leaf
            children_index_indice: 0,
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

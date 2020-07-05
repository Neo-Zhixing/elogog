use std::iter::Fuse;

#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct TupleStrip<T>
    where T: Iterator {
    iter: Fuse<T>,
    last: Option<T::Item>,
}

pub fn tuple_strip<T>(iter: T) -> TupleStrip<T>
    where T: Iterator
{
    TupleStrip {
        iter: iter.fuse(),
        last: None,
    }
}

impl<T> Iterator for TupleStrip<T>
    where T: Iterator,
        T::Item: Copy
{
    type Item = (T::Item, T::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(last) = self.last {
            if let Some(current) = self.iter.next() {
                self.last = Some(current);
                return Some((last, current));
            }
        } else {
            if let Some(last) = self.iter.next() {
                if let Some(current) = self.iter.next() {
                    self.last = Some(current);
                    return Some((last, current));
                }
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Should be exactly 1 less than parent
        let (mut min, mut max) = self.iter.size_hint();
        if min > 0 {
            min -= 1;
        }
        if let Some(max_n) = max {
            if max_n > 0 {
                max = Some(max_n - 1);
            }
        }
        (min, max)
    }
}

impl<T> ExactSizeIterator for TupleStrip<T> where T: ExactSizeIterator, T::Item: Copy {}


pub trait IterUtil : Iterator {
    #[inline]
    fn tuple_strip(self) -> TupleStrip<Self>
        where Self: Sized
    {
        tuple_strip(self)
    }
}

impl<T> IterUtil for T where T: Iterator { }

#[cfg(test)]
mod tests {
    use super::TupleStrip;
    use super::tuple_strip;
    use super::IterUtil;
    #[test]
    fn test_nums() {
        let num_runs = 10;
        let mut iter = (0..=num_runs).tuple_strip();
        for i in 0..num_runs {
            assert_eq!(iter.next(), Some((i, i+1)));
        }
    }

    #[test]
    fn test_small_iters() {
        let mut iter = (0..=1).tuple_strip();
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), None);
        let mut iter = (0..1).tuple_strip();
        assert_eq!(iter.next(), None);
        let mut iter = (0..=2).tuple_strip();
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), Some((1, 2)));
        assert_eq!(iter.next(), None);
    }
}

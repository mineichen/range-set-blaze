use crate::{Integer, NonZeroRange, SortedDisjoint, SortedStarts};
use core::{
    cmp::{max, min},
    iter::FusedIterator,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct UnsortedDisjoint<T, I> {
    iter: I,
    option_range: Option<NonZeroRange<T>>,
    min_value_plus_2: T,
}

impl<T, I> UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>>,
{
    #[inline]
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            option_range: None,
            min_value_plus_2: T::min_value().add_one().add_one(),
        }
    }
}

impl<T, I> FusedIterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>> + FusedIterator,
{
}

impl<T, I> Iterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>>,
{
    type Item = NonZeroRange<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(range) = self.iter.next() else {
                return self.option_range.take();
            };
            let (next_start, next_end) = (range.start, range.end);
            let Some(self_range) = self.option_range else {
                self.option_range = Some(range);
                continue;
            };

            let (self_start, self_end) = (self_range.start, self_range.end);
            if (next_start >= self.min_value_plus_2 && self_end <= next_start.sub_one().sub_one())
                || (self_start >= self.min_value_plus_2
                    && next_end <= self_start.sub_one().sub_one())
            {
                let result = Some(self_range);
                self.option_range = Some(range);
                return result;
            }
            self.option_range = Some(unsafe {
                NonZeroRange::new_unchecked(min(self_start, next_start)..max(self_end, next_end))
            });
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = min(lower, 1);
        if self.option_range.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    iter: I,
    len: <T as Integer>::SafeLen,
}

impl<T, I> SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>> + SortedDisjoint<T>,
{
    #[inline]
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            len: T::SafeLen::zero(),
        }
    }
}

impl<T: Integer, I> SortedDisjointWithLenSoFar<T, I>
where
    I: SortedDisjoint<T>,
{
    pub(crate) const fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }
}

impl<T: Integer, I> FusedIterator for SortedDisjointWithLenSoFar<T, I> where
    I: SortedDisjoint<T> + FusedIterator
{
}

impl<T: Integer, I> Iterator for SortedDisjointWithLenSoFar<T, I>
where
    I: SortedDisjoint<T>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = self.iter.next() {
            let (start, end) = (range.start, range.end);
            debug_assert!(start < end);
            self.len += T::safe_len(&(start..=end.sub_one()));
            Some((start, end.sub_one()))
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
/// Gives any iterator of ranges the [`SortedStarts`] trait without any checking.
pub struct AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>> + FusedIterator,
{
    pub(crate) iter: I,
}

impl<T, I> FusedIterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>> + FusedIterator,
{
}

impl<T: Integer, I> SortedStarts<T> for AssumeSortedStarts<T, I> where
    I: Iterator<Item = NonZeroRange<T>> + FusedIterator
{
}

impl<T, I> AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>> + FusedIterator,
{
    #[inline]
    pub fn new<J: IntoIterator<IntoIter = I>>(iter: J) -> Self {
        Self {
            iter: iter.into_iter(),
        }
    }
}

impl<T, I> Iterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = NonZeroRange<T>> + FusedIterator,
{
    type Item = NonZeroRange<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

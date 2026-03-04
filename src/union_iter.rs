use crate::merge::KMerge;
use crate::union_iter_map::SortedStartsInVec;
use crate::unsorted_disjoint::UnsortedDisjoint;
use crate::{AssumeSortedStarts, Merge, SortedDisjoint, SortedStarts, UnionKMerge};
use crate::{Integer, UnionMerge};
use core::cmp::max;
use crate::NonZeroRange;
use core::iter::FusedIterator;
use itertools::Itertools;

/// This `struct` is created by the [`union`] method on [`SortedStarts`]. See [`union`]'s
/// documentation for more.
///
/// [`SortedStarts`]: crate::SortedStarts
/// [`union`]: crate::SortedDisjoint::union
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnionIter<T, SS>
where
    T: Integer,
    SS: SortedStarts<T>,
{
    iter: SS,
    option_range: Option<NonZeroRange<T>>,
}

impl<T, I> Iterator for UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    type Item = NonZeroRange<T>;

    fn next(&mut self) -> Option<NonZeroRange<T>> {
        loop {
            let Some(range) = self.iter.next() else {
                return self.option_range.take();
            };

            let (start, end) = (range.start, range.end);
            debug_assert!(start < end); // real assert

            let Some(current_range) = self.option_range.take() else {
                self.option_range = Some(range);
                continue;
            };

            let (current_start, current_end) = (current_range.start, current_range.end);
            debug_assert!(current_start <= start); // real assert
            if start <= current_end {
                self.option_range = Some(unsafe {
                    NonZeroRange::new_unchecked(current_start..max(current_end, end))
                });
                continue;
            }

            self.option_range = Some(range);
            return Some(current_range);
        }
    }
}

impl<T, I> UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    /// Creates a new [`UnionIter`] from zero or more [`SortedStarts`] iterators. See [`UnionIter`] for more details and examples.
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            option_range: None,
        }
    }
}

impl<T, L, R> UnionMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    #[inline]
    pub(crate) fn new2(left: L, right: R) -> Self {
        let iter: Merge<T, L, R> = Merge::new(left, right);
        Self::new(iter)
    }
}

impl<T, J> UnionKMerge<T, J>
where
    T: Integer,
    J: SortedDisjoint<T>,
{
    #[inline]
    pub(crate) fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMerge::new(k);
        Self::new(iter)
    }
}

// from iter (T, VR) to UnionIter
impl<T> FromIterator<NonZeroRange<T>> for UnionIter<T, SortedStartsInVec<T>>
where
    T: Integer,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = NonZeroRange<T>>,
    {
        let iter = iter.into_iter();
        let iter = UnsortedDisjoint::new(iter);
        let iter = iter.sorted_by(|a, b| a.start.cmp(&b.start));
        let iter = AssumeSortedStarts::new(iter);
        Self::new(iter)
    }
}

impl<T, I> FusedIterator for UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T> + FusedIterator,
{
}

use crate::sym_diff_iter_map::UsizeExtensions;
use crate::{
    Integer, Merge, SortedDisjoint, SortedStarts, SymDiffKMerge, SymDiffMerge, merge::KMerge,
};
use crate::NonZeroRange;
use alloc::collections::BinaryHeap;
use core::{cmp::Reverse, iter::FusedIterator};

/// This `struct` is created by the [`symmetric_difference`] method on [`SortedDisjoint`]. See [`symmetric_difference`]'s
/// documentation for more.
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`symmetric_difference`]: crate::SortedDisjointMap::symmetric_difference
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    iter: I,
    start_or_min_value: T,
    end_heap: BinaryHeap<Reverse<T>>,
    next_again: Option<NonZeroRange<T>>,
    gather: Option<NonZeroRange<T>>,
}

impl<T, I> FusedIterator for SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
}

impl<T, I> Iterator for SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    type Item = NonZeroRange<T>;

    fn next(&mut self) -> Option<NonZeroRange<T>> {
        loop {
            let count = self.end_heap.len();
            let Some(next_range) = self.next_again.take().or_else(|| self.iter.next()) else {
                if count == 0 {
                    return self.gather.take();
                }

                let end = self
                    .end_heap
                    .pop()
                    .expect("Real Assert: the workspace is not empty")
                    .0;
                self.remove_same_end(end);
                let result = unsafe {
                    NonZeroRange::new_unchecked(self.start_or_min_value..end)
                };
                if !self.end_heap.is_empty() {
                    self.start_or_min_value = end;
                }
                if let Some(result) = self.process(count.is_odd(), result) {
                    return result;
                }
                continue;
            };

            let (next_start, next_end) = (next_range.start, next_range.end);
            if count == 0 || self.start_or_min_value == next_start {
                self.start_or_min_value = next_start;
                self.end_heap.push(Reverse(next_end));
                continue;
            }

            let end = self
                .end_heap
                .peek()
                .expect("Real Assert: The workspace has a first chunk.")
                .0;
            if next_start <= end {
                let result = unsafe {
                    NonZeroRange::new_unchecked(self.start_or_min_value..next_start)
                };
                self.start_or_min_value = next_start;
                self.end_heap.push(Reverse(next_end));
                if let Some(result) = self.process(count.is_odd(), result) {
                    return result;
                }
                continue;
            }

            self.remove_same_end(end);
            let result = unsafe {
                NonZeroRange::new_unchecked(self.start_or_min_value..end)
            };
            if self.end_heap.is_empty() {
                self.start_or_min_value = next_start;
                self.end_heap.push(Reverse(next_end));
                if let Some(result) = self.process(count.is_odd(), result) {
                    return result;
                }
                continue;
            }

            self.start_or_min_value = end;
            self.next_again = Some(next_range);
            if let Some(result) = self.process(count.is_odd(), result) {
                return result;
            }
        }
    }
}

impl<T, I> SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    #[inline]
    fn remove_same_end(&mut self, end: T) {
        while let Some(end2) = self.end_heap.peek() {
            if end2.0 == end {
                self.end_heap.pop();
            } else {
                break;
            }
        }
    }

    #[inline]
    #[allow(clippy::option_option)]
    fn process(
        &mut self,
        keep: bool,
        next: NonZeroRange<T>,
    ) -> Option<Option<NonZeroRange<T>>> {
        if !keep {
            return None;
        }
        let Some(gather) = self.gather.take() else {
            self.gather = Some(next);
            return None;
        };

        let (next_start, next_end) = (next.start, next.end);
        let (gather_start, gather_end) = (gather.start, gather.end);

        debug_assert!(gather_end <= next_start); // real assert

        if gather_end == next_start {
            self.gather = Some(unsafe {
                NonZeroRange::new_unchecked(gather_start..next_end)
            });
            return None;
        }

        self.gather = Some(next);
        Some(Some(gather))
    }

    #[inline]
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            start_or_min_value: T::min_value(),
            end_heap: BinaryHeap::with_capacity(10),
            next_again: None,
            gather: None,
        }
    }
}

impl<T, L, R> SymDiffMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    #[inline]
    pub(crate) fn new2(left: L, right: R) -> Self {
        let iter = Merge::new(left, right);
        Self::new(iter)
    }
}

impl<T, J> SymDiffKMerge<T, J>
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

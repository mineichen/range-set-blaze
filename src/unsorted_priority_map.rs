use crate::map::ValueRef;
use crate::range_values::ExpectDebugUnwrapRelease;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{Integer, NonZeroRange, map::EndValue, sorted_disjoint_map::SortedDisjointMap};
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (NonZeroRange<T>, VR)>,
{
    iter: I,
    option_priority: Option<Priority<T, VR>>,
    priority_number: usize,
}

impl<T, VR, I> UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (NonZeroRange<T>, VR)>,
{
    #[inline]
    pub(crate) fn new(into_iter: I) -> Self {
        Self {
            iter: into_iter,
            option_priority: None,
            priority_number: 0,
        }
    }
}

impl<T, VR, I> FusedIterator for UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (NonZeroRange<T>, VR)>,
{
}

impl<T, VR, I> Iterator for UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (NonZeroRange<T>, VR)>,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(next_range_value) = self.iter.next() else {
                return self.option_priority.take();
            };
            let next_priority = Priority::new(next_range_value, self.priority_number);
            self.priority_number = self
                .priority_number
                .checked_add(1)
                .expect_debug_unwrap_release("underflow");

            let (next_start, next_end) = next_priority.start_and_end();
            if next_start >= next_end {
                continue;
            }

            let Some(mut current_priority) = self.option_priority.take() else {
                self.option_priority = Some(next_priority);
                continue;
            };

            let (current_start, current_end) = current_priority.start_and_end();
            if current_priority.value().borrow() != next_priority.value().borrow()
                || current_end < next_start
                || next_end < current_start
            {
                self.option_priority = Some(next_priority);
                return Some(current_priority);
            }

            current_priority.set_range(unsafe {
                NonZeroRange::new_unchecked(min(current_start, next_start)..max(current_end, next_end))
            });
            self.option_priority = Some(current_priority);
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = min(lower, 1);
        if self.option_priority.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: I,
    len: <T as Integer>::SafeLen,
    phantom: PhantomData<VR>,
}

impl<T, VR, I> SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    pub(crate) const fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }

    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            len: <T as Integer>::SafeLen::zero(),
            phantom: PhantomData,
        }
    }
}

impl<T, VR, I> FusedIterator for SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, I> Iterator for SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = (T, EndValue<T, VR::Target>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((range, value)) = self.iter.next() {
            let (start, end) = (range.start, range.end);
            debug_assert!(start < end);
            let inclusive_end = end.sub_one();
            self.len += T::safe_len(&(start..=inclusive_end));
            let end_value = EndValue {
                end: inclusive_end,
                value: value.into_value(),
            };
            Some((start, end_value))
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
/// Used internally by [`UnionIterMap`] and [`SymDiffIterMap`].
///
/// [`UnionIterMap`]: crate::UnionIterMap
/// [`SymDiffIterMap`]: crate::SymDiffIterMap
pub struct AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
    iter: I,
}

impl<T, VR, I> PrioritySortedStartsMap<T, VR> for AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
}

impl<T, VR, I> AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
    pub(crate) const fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<T, VR, I> FusedIterator for AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
}

impl<T, VR, I> Iterator for AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

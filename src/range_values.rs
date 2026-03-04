use crate::{
    Integer,
    map::ValueRef,
    sorted_disjoint_map::{Priority, PrioritySortedStartsMap},
};
use crate::NonZeroRange;
use alloc::{collections::btree_map, rc::Rc};
use core::{iter::FusedIterator, marker::PhantomData};

use crate::{map::EndValue, sorted_disjoint_map::SortedDisjointMap};

/// This `struct` is created by the [`range_values`] method on [`RangeMapBlaze`]. See [`range_values`]'s
/// documentation for more. Double-ended.
///
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
/// [`range_values`]: crate::RangeMapBlaze::range_values
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::module_name_repetitions)]
pub struct RangeValuesIter<'a, T: Integer, V: Eq + Clone> {
    iter: btree_map::Iter<'a, T, EndValue<T, V>>,
}

impl<'a, T: Integer, V: Eq + Clone> RangeValuesIter<'a, T, V> {
    #[inline]
    pub(crate) fn new(map: &'a btree_map::BTreeMap<T, EndValue<T, V>>) -> Self {
        Self { iter: map.iter() }
    }
}

impl<T: Integer, V: Eq + Clone> ExactSizeIterator for RangeValuesIter<'_, T, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer, V: Eq + Clone> FusedIterator for RangeValuesIter<'_, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for RangeValuesIter<'a, T, V>
where
    T: Integer,
    V: Eq + Clone + 'a,
{
    type Item = (NonZeroRange<T>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| {
            (
                unsafe { NonZeroRange::new_unchecked(*start..end_value.end.add_one()) },
                &end_value.value,
            )
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, V> DoubleEndedIterator for RangeValuesIter<'a, T, V>
where
    T: Integer,
    V: Eq + Clone + 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(start, end_value)| {
            (
                unsafe { NonZeroRange::new_unchecked(*start..end_value.end.add_one()) },
                &end_value.value,
            )
        })
    }
}

/// This `struct` is created by the [`into_range_values`] method on [`RangeMapBlaze`]. See [`into_range_values`]'s
/// documentation for more. Double-ended.
///
/// Not clonable because `btree_map::IntoIter` is not clonable.
///
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
/// [`into_range_values`]: crate::RangeMapBlaze::into_range_values
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct IntoRangeValuesIter<T: Integer, V: Eq + Clone> {
    iter: btree_map::IntoIter<T, EndValue<T, V>>,
}

impl<T: Integer, V: Eq + Clone> IntoRangeValuesIter<T, V> {
    #[inline]
    pub(crate) fn new(map: btree_map::BTreeMap<T, EndValue<T, V>>) -> Self {
        Self {
            iter: map.into_iter(),
        }
    }
}

impl<T: Integer, V: Eq + Clone> ExactSizeIterator for IntoRangeValuesIter<T, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer, V: Eq + Clone> FusedIterator for IntoRangeValuesIter<T, V> {}

impl<T: Integer, V: Eq + Clone> Iterator for IntoRangeValuesIter<T, V> {
    type Item = (NonZeroRange<T>, Rc<V>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| {
            let range = unsafe { NonZeroRange::new_unchecked(start..end_value.end.add_one()) };
            let value = Rc::new(end_value.value);
            (range, value)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer, V: Eq + Clone> DoubleEndedIterator for IntoRangeValuesIter<T, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(start, end_value)| {
            let range = unsafe { NonZeroRange::new_unchecked(start..end_value.end.add_one()) };
            let value = Rc::new(end_value.value);
            (range, value)
        })
    }
}

/// This `struct` is created by the [`ranges`] method on [`RangeMapBlaze`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
/// [`ranges`]: crate::RangeMapBlaze::ranges
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MapRangesIter<'a, T: Integer, V: Eq + Clone> {
    iter: btree_map::Iter<'a, T, EndValue<T, V>>,
    gather: Option<NonZeroRange<T>>,
}

impl<'a, T: Integer, V: Eq + Clone> MapRangesIter<'a, T, V> {
    pub(crate) const fn new(iter: btree_map::Iter<'a, T, EndValue<T, V>>) -> Self {
        MapRangesIter { iter, gather: None }
    }
}

impl<T: Integer, V: Eq + Clone> FusedIterator for MapRangesIter<'_, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for MapRangesIter<'a, T, V>
where
    T: Integer,
    V: Eq + Clone + 'a,
{
    type Item = NonZeroRange<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((start, end_value)) = self.iter.next() else {
                return self.gather.take();
            };

            let (start_next, end_next) = (*start, end_value.end.add_one());
            debug_assert!(start_next < end_next);

            let Some(gather) = self.gather.take() else {
                self.gather =
                    Some(unsafe { NonZeroRange::new_unchecked(start_next..end_next) });
                continue;
            };

            if gather.end == start_next {
                self.gather = Some(unsafe {
                    NonZeroRange::new_unchecked(gather.start..end_next)
                });
                continue;
            }

            self.gather =
                Some(unsafe { NonZeroRange::new_unchecked(start_next..end_next) });
            return Some(gather);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

/// This `struct` is created by the [`into_ranges`] method on [`RangeMapBlaze`]. See [`into_ranges`]'s
/// documentation for more.
///
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
/// [`into_ranges`]: crate::RangeMapBlaze::into_ranges
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct MapIntoRangesIter<T: Integer, V: Eq + Clone> {
    iter: btree_map::IntoIter<T, EndValue<T, V>>,
    gather: Option<NonZeroRange<T>>,
}

impl<T: Integer, V: Eq + Clone> MapIntoRangesIter<T, V> {
    pub(crate) const fn new(iter: btree_map::IntoIter<T, EndValue<T, V>>) -> Self {
        Self { iter, gather: None }
    }
}

impl<T: Integer, V: Eq + Clone> FusedIterator for MapIntoRangesIter<T, V> {}

impl<T: Integer, V: Eq + Clone> Iterator for MapIntoRangesIter<T, V> {
    type Item = NonZeroRange<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((start_next, end_value)) = self.iter.next() else {
                return self.gather.take();
            };

            let end_next = end_value.end.add_one();
            debug_assert!(start_next < end_next);

            let Some(gather) = self.gather.take() else {
                self.gather =
                    Some(unsafe { NonZeroRange::new_unchecked(start_next..end_next) });
                continue;
            };

            if gather.end == start_next {
                self.gather = Some(unsafe {
                    NonZeroRange::new_unchecked(gather.start..end_next)
                });
                continue;
            }

            self.gather =
                Some(unsafe { NonZeroRange::new_unchecked(start_next..end_next) });
            return Some(gather);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

/// This `struct` is used internally.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::module_name_repetitions)]
pub struct RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: I,
    gather: Option<NonZeroRange<T>>,
    phantom: PhantomData<VR>,
}

impl<T, VR, I> FusedIterator for RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, I> RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    /// Creates a new `RangeValuesToRangesIter` from an existing sorted disjoint map iterator.
    /// `option_ranges` is initialized as `None` by default.
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            gather: None,
            phantom: PhantomData,
        }
    }
}

impl<T, VR, I> Iterator for RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = NonZeroRange<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(next_range_value) = self.iter.next() else {
                return self.gather.take();
            };
            let next_range = next_range_value.0;

            let Some(gather) = self.gather.take() else {
                self.gather = Some(next_range);
                continue;
            };

            if gather.end == next_range.start {
                self.gather = Some(unsafe {
                    NonZeroRange::new_unchecked(gather.start..next_range.end)
                });
                continue;
            }

            self.gather = Some(next_range);
            return Some(gather);
        }
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) trait ExpectDebugUnwrapRelease<T> {
    fn expect_debug_unwrap_release(self, msg: &str) -> T;
}

#[allow(unused_variables)]
impl<T> ExpectDebugUnwrapRelease<T> for Option<T> {
    fn expect_debug_unwrap_release(self, msg: &str) -> T {
        #[cfg(debug_assertions)]
        {
            self.expect(msg)
        }
        #[cfg(not(debug_assertions))]
        {
            self.unwrap()
        }
    }
}

#[expect(clippy::redundant_pub_crate)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub(crate) struct SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: I,
    priority_number: usize,
    phantom: PhantomData<(T, VR)>,
}

impl<T, VR, I> FusedIterator for SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, I> Iterator for SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|range_value| Priority::new(range_value, self.priority_number))
    }
}

impl<T, VR, I> SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    pub(crate) const fn new(iter: I, priority: usize) -> Self {
        Self {
            iter,
            priority_number: priority,
            phantom: PhantomData,
        }
    }
}

impl<T, VR, I> PrioritySortedStartsMap<T, VR> for SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

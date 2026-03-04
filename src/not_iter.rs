use core::iter::FusedIterator;

use crate::{Integer, NonZeroRange, SortedDisjoint};

/// The output of [`SortedDisjoint::complement`] and [`SortedDisjointMap::complement_with`].
///
/// [`SortedDisjointMap::complement_with`]: trait.SortedDisjointMap.html#method.complement_with
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    iter: I,
    start_not: T,
    next_time_return_none: bool,
}

impl<T, I> NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    #[inline]
    pub(crate) fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = NonZeroRange<T>, IntoIter = I>,
    {
        Self {
            iter: iter.into_iter(),
            start_not: T::min_value(),
            next_time_return_none: false,
        }
    }
}

impl<T, I> FusedIterator for NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T> + FusedIterator,
{
}

// Note: DoubleEndedIterator is not easily implementable for NotIter because
// it would require complex tracking of the "flipped" nature of the NotIter
// in reverse order.

impl<T, I> Iterator for NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    type Item = NonZeroRange<T>;
    fn next(&mut self) -> Option<NonZeroRange<T>> {
        debug_assert!(T::min_value() <= T::max_value()); // real assert
        if self.next_time_return_none {
            return None;
        }
        let next_item = self.iter.next();
        if let Some(range) = next_item {
            let (start, end) = (range.start, range.end);
            debug_assert!(start < end);
            if self.start_not < start {
                let result =
                    Some(unsafe { NonZeroRange::new_unchecked(self.start_not..start) });
                self.start_not = end;
                result
            } else {
                self.start_not = end;
                self.next() // will recurse at most once
            }
        } else {
            self.next_time_return_none = true;
            if let Some(end) = T::max_value().checked_add_one() {
                if self.start_not <= T::max_value() {
                    Some(unsafe { NonZeroRange::new_unchecked(self.start_not..end) })
                } else {
                    None
                }
            } else if self.start_not < T::max_value() {
                Some(unsafe {
                    NonZeroRange::new_unchecked(self.start_not..T::max_value())
                })
            } else {
                None
            }
        }
    }

    // We could have one less or one more than the iter.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let low = low.saturating_sub(1);
        let high = high.map(|high| high.saturating_add(1));
        (low, high)
    }
}

// FUTURE define Not, etc on DynSortedDisjoint
